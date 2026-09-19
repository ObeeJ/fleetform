use anyhow::{anyhow, Result};
use aws_sdk_ec2::types::{Filter, IpPermission, IpRange, ResourceType, Tag, TagSpecification};
use aws_sdk_ec2::Client as Ec2Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::hcl::{parse_blocks, Block};
use crate::state::{ManagedResource, State};
use crate::terminal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Create,
    NoOp,
    Destroy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedChange {
    pub address: String,
    pub resource_type: String,
    pub name: String,
    pub action: Action,
    pub ami: Option<String>,
    pub instance_type: Option<String>,
    pub bucket: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub changes: Vec<PlannedChange>,
    pub add: usize,
    pub change: usize,
    pub unchanged: usize,
    pub destroy: usize,
    pub live: bool,
}

impl Plan {
    pub fn is_empty(&self) -> bool {
        self.add == 0 && self.change == 0 && self.destroy == 0
    }
}

pub fn live_enabled() -> bool {
    matches!(
        std::env::var("FLEETFORM_LIVE").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

pub fn workspace_name() -> String {
    std::env::var("FLEETFORM_WORKSPACE").unwrap_or_else(|_| "default".into())
}

pub fn load_desired() -> Result<Vec<Block>> {
    let path = std::path::Path::new("main.tf");
    if !path.exists() {
        return Err(anyhow!("No main.tf in the working directory"));
    }
    let contents = std::fs::read_to_string(path)?;
    crate::hcl::validate_hcl_syntax(&contents)?;
    parse_blocks(&contents)
}

fn address_of(block: &Block) -> Option<String> {
    if block.block_type == "resource" && block.labels.len() == 2 {
        Some(format!("{}.{}", block.labels[0], block.labels[1]))
    } else {
        None
    }
}

fn find_managed<'a>(state: &'a State, address: &str) -> Option<&'a ManagedResource> {
    state.managed.iter().find(|r| r.address == address)
}

pub fn build_plan(desired: &[Block], state: &State) -> Plan {
    let live = live_enabled();
    let mut changes = Vec::new();
    for block in desired {
        let Some(address) = address_of(block) else {
            continue;
        };
        let resource_type = block.labels[0].clone();
        let name = block.labels[1].clone();
        let existing = find_managed(state, &address);
        let (action, note) = match existing {
            Some(r) if r.status == "running" || r.status == "available" => (
                Action::NoOp,
                format!(
                    "already {} ({})",
                    r.status,
                    r.id.clone().unwrap_or_default()
                ),
            ),
            // plan never calls AWS, so it cannot know whether the recorded id
            // still exists. Say what apply will actually do rather than
            // promising a recreate it may not perform.
            Some(r) if r.id.is_some() => (
                Action::Create,
                format!(
                    "prior id {} recorded as {}; apply adopts it if it still exists, otherwise creates",
                    r.id.clone().unwrap_or_default(),
                    r.status
                ),
            ),
            Some(r) if r.status == "planned" => {
                (Action::Create, "recorded locally; not yet in AWS".into())
            }
            _ => (Action::Create, "will be created".into()),
        };
        // A key pair and security group are implied by an instance rather than
        // declared, so say so here instead of letting apply surprise anyone.
        let note = if resource_type == "aws_instance" && action == Action::Create {
            format!("{}; also creates an SSH key pair and security group", note)
        } else {
            note
        };
        changes.push(PlannedChange {
            address,
            resource_type,
            name,
            action,
            ami: block.attributes.get("ami").cloned(),
            instance_type: block.attributes.get("instance_type").cloned(),
            bucket: block.attributes.get("bucket").cloned(),
            note,
        });
    }
    let add = changes
        .iter()
        .filter(|c| c.action == Action::Create)
        .count();
    let unchanged = changes.iter().filter(|c| c.action == Action::NoOp).count();
    let destroy = changes
        .iter()
        .filter(|c| c.action == Action::Destroy)
        .count();
    Plan {
        changes,
        add,
        // There is no in-place update action yet: a resource is created, left
        // alone, or destroyed. Counting no-ops here made a converged stack
        // report "N to change" and never look empty.
        change: 0,
        unchanged,
        destroy,
        live,
    }
}

/// AWS names allow a narrow character set; an HCL address like
/// `aws_instance.example` has to be flattened before it can be one.
fn sanitize(component: &str) -> String {
    component
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

pub fn key_pair_name(workspace: &str, address: &str) -> String {
    format!("fleetform-{}-{}", sanitize(workspace), sanitize(address))
}

pub fn security_group_name(workspace: &str, address: &str) -> String {
    format!("{}-sg", key_pair_name(workspace, address))
}

/// Where a managed key pair and security group are recorded in state. They are
/// implied by an instance rather than declared in HCL, so they get a suffixed
/// address instead of one of their own.
pub fn key_pair_address(instance_address: &str) -> String {
    format!("{}#key", instance_address)
}

pub fn security_group_address(instance_address: &str) -> String {
    format!("{}#sg", instance_address)
}

pub fn cidr_from_public_ip(raw: &str) -> Result<String> {
    let ip = raw.trim();
    let octets: Vec<&str> = ip.split('.').collect();
    let valid = octets.len() == 4
        && octets
            .iter()
            .all(|o| !o.is_empty() && o.parse::<u8>().is_ok());
    if !valid {
        return Err(anyhow!("expected an IPv4 address, got {:?}", ip));
    }
    Ok(format!("{}/32", ip))
}

async fn detect_public_ip() -> Result<String> {
    let body = reqwest::Client::new()
        .get("https://checkip.amazonaws.com")
        .timeout(Duration::from_secs(10))
        .send()
        .await?
        .text()
        .await?;
    cidr_from_public_ip(&body)
}

/// SSH ingress for a managed security group. Defaults to just this machine so
/// a fresh apply is reachable by whoever ran it without putting port 22 on the
/// public internet. `ssh_cidr` on the resource overrides it.
pub async fn resolve_ssh_cidr(configured: Option<&str>) -> Result<String> {
    if let Some(cidr) = configured {
        let cidr = cidr.trim();
        if !cidr.contains('/') {
            return Err(anyhow!(
                "ssh_cidr must include a prefix length, e.g. \"{}/32\"",
                cidr
            ));
        }
        if cidr == "0.0.0.0/0" {
            terminal::warn("ssh_cidr is 0.0.0.0/0: port 22 will be open to the internet.");
        }
        return Ok(cidr.to_string());
    }
    let cidr = detect_public_ip().await.map_err(|e| {
        anyhow!(
            "could not detect this machine's public IP to scope SSH access ({}). \
             Set ssh_cidr on the resource, e.g. ssh_cidr = \"203.0.113.4/32\", \
             or \"0.0.0.0/0\" to allow the internet.",
            e
        )
    })?;
    terminal::info(&format!("Scoping SSH to this machine: {}", cidr));
    Ok(cidr)
}

/// Write state to disk now. Every cloud resource that exists has to be recorded
/// before the next API call: a timeout, an error or Ctrl-C in between leaves a
/// billed resource running with nothing on disk pointing at it.
async fn persist(state: &State) -> Result<()> {
    crate::state::save(state).await
}

fn write_private_key(key_name: &str, material: &str) -> Result<std::path::PathBuf> {
    let dir = std::path::Path::new(".fleetform").join("keys");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.pem", key_name));
    std::fs::write(&path, material)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // ssh refuses to use a key that other local users can read.
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(path)
}

/// Cloud ids that make a non-live destroy unsafe. Clearing local state while
/// these exist orphans resources that keep running and keep billing, with no
/// recorded id left to find them again.
pub fn blocking_cloud_ids(live: bool, state: &State) -> Vec<String> {
    if live {
        return Vec::new();
    }
    state.managed.iter().filter_map(|r| r.id.clone()).collect()
}

pub struct Engine {
    ec2: Option<Ec2Client>,
}

impl Engine {
    pub async fn connect() -> Result<Self> {
        let cfg = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .load()
            .await;
        Ok(Self {
            ec2: Some(Ec2Client::new(&cfg)),
        })
    }

    pub fn dry() -> Self {
        Self { ec2: None }
    }

    fn ec2(&self) -> Result<&Ec2Client> {
        self.ec2
            .as_ref()
            .ok_or_else(|| anyhow!("AWS client not initialized"))
    }

    pub async fn apply_plan(
        &self,
        plan: &Plan,
        desired: &[Block],
        state: &mut State,
    ) -> Result<()> {
        if !plan.live {
            terminal::warn("FLEETFORM_LIVE is not set. Recording planned resources only.");
            terminal::warn("Export FLEETFORM_LIVE=1 to launch a real machine.");
            for change in &plan.changes {
                if change.action == Action::Create {
                    upsert(
                        state,
                        ManagedResource {
                            address: change.address.clone(),
                            resource_type: change.resource_type.clone(),
                            name: change.name.clone(),
                            id: None,
                            status: "planned".into(),
                            public_ip: None,
                        },
                    );
                }
            }
            return Ok(());
        }

        for change in &plan.changes {
            if change.action != Action::Create {
                continue;
            }
            let Some(block) = desired
                .iter()
                .find(|b| address_of(b).as_deref() == Some(change.address.as_str()))
            else {
                continue;
            };
            match change.resource_type.as_str() {
                "aws_instance" => self.apply_instance(block, state).await?,
                "aws_s3_bucket" => {
                    terminal::warn(&format!(
                        "S3 create for {} skipped in Phase 1. EC2 is the launch path.",
                        change.address
                    ));
                    upsert(
                        state,
                        ManagedResource {
                            address: change.address.clone(),
                            resource_type: change.resource_type.clone(),
                            name: change.name.clone(),
                            id: None,
                            status: "skipped".into(),
                            public_ip: None,
                        },
                    );
                }
                other => terminal::warn(&format!("unsupported resource type: {}", other)),
            }
        }
        Ok(())
    }

    async fn apply_instance(&self, block: &Block, state: &mut State) -> Result<()> {
        let address = address_of(block).ok_or_else(|| anyhow!("invalid instance block"))?;
        let name = block.labels[1].clone();

        if let Some(existing) = find_managed(state, &address) {
            if let Some(id) = existing.id.clone() {
                if let Some(mut live) = self.describe(&id).await? {
                    live.address = address.clone();
                    live.name = name.clone();
                    live.resource_type = "aws_instance".into();
                    let status = live.status.clone();
                    match status.as_str() {
                        "running" => {
                            terminal::success(&format!(
                                "{} already running as {} {}",
                                address,
                                id,
                                live.public_ip.clone().unwrap_or_default()
                            ));
                            upsert(state, live);
                            persist(state).await?;
                            return Ok(());
                        }
                        // Still booting from an earlier apply that did not get
                        // to finish. Waiting is right; launching a second
                        // machine is how the first one gets stranded.
                        "pending" => {
                            terminal::info(&format!(
                                "{} is already {} as {}; waiting rather than launching another",
                                address, status, id
                            ));
                            let live = self.wait_running(&address, &name, &id).await?;
                            upsert(state, live);
                            persist(state).await?;
                            return Ok(());
                        }
                        // Genuinely gone, so creating a replacement is correct.
                        "terminated" | "shutting-down" => {}
                        // Stopped or stopping: still a billed machine that this
                        // address owns. Launching another would leave it with
                        // no record once upsert replaced its id.
                        _ => {
                            upsert(state, live);
                            persist(state).await?;
                            return Err(anyhow!(
                                "{} is recorded as {}, which is {} in AWS. Refusing to launch \
                                 a second machine for the same address - start or terminate \
                                 {} first.",
                                address,
                                id,
                                status,
                                id
                            ));
                        }
                    }
                }
            }
        }

        let instance_type = block
            .attributes
            .get("instance_type")
            .cloned()
            .unwrap_or_else(|| "t3.micro".into());
        let ami = resolve_ami(self.ec2()?, block.attributes.get("ami").map(|s| s.as_str())).await?;

        let key_name = self.ensure_key_pair(&address, state).await?;
        let ssh_cidr =
            resolve_ssh_cidr(block.attributes.get("ssh_cidr").map(|s| s.as_str())).await?;
        let security_group = self
            .ensure_security_group(&address, &ssh_cidr, state)
            .await?;

        terminal::info(&format!(
            "Launching {} ami={} type={}",
            address, ami, instance_type
        ));

        let tags = TagSpecification::builder()
            .resource_type(ResourceType::Instance)
            .tags(
                Tag::builder()
                    .key("Name")
                    .value(format!("fleetform-{}", name))
                    .build(),
            )
            .tags(Tag::builder().key("ManagedBy").value("fleetform").build())
            .tags(
                Tag::builder()
                    .key("fleetform:address")
                    .value(&address)
                    .build(),
            )
            .build();

        let result = self
            .ec2()?
            .run_instances()
            .image_id(&ami)
            .instance_type(instance_type.as_str().into())
            .min_count(1)
            .max_count(1)
            .key_name(&key_name)
            .security_group_ids(&security_group)
            .client_token(&client_token(&address))
            .tag_specifications(tags)
            .send()
            .await
            .map_err(|e| anyhow!("RunInstances failed: {}", e))?;

        let id = result
            .instances()
            .first()
            .and_then(|i| i.instance_id())
            .ok_or_else(|| anyhow!("RunInstances returned no instance id"))?
            .to_string();

        // RunInstances has already created a billed machine. Record it before
        // waiting: wait_running can time out or fail, and until this is on disk
        // the instance is running with nothing naming its id.
        upsert(
            state,
            ManagedResource {
                address: address.clone(),
                resource_type: "aws_instance".into(),
                name: name.clone(),
                id: Some(id.clone()),
                status: "pending".into(),
                public_ip: None,
            },
        );
        persist(state).await?;

        terminal::info(&format!("Waiting for {} to reach running...", id));
        let live = self.wait_running(&address, &name, &id).await?;
        terminal::success(&format!(
            "Machine running: {} id={} ip={}",
            address,
            id,
            live.public_ip.clone().unwrap_or_else(|| "none".into())
        ));
        upsert(state, live);
        persist(state).await?;
        Ok(())
    }

    /// Create (or adopt) the SSH key pair for an instance. The private key is
    /// only ever returned by CreateKeyPair, so it is written to disk here or
    /// it is gone for good.
    async fn ensure_key_pair(&self, address: &str, state: &mut State) -> Result<String> {
        let key_name = key_pair_name(&workspace_name(), address);
        let record = |state: &mut State| {
            upsert(
                state,
                ManagedResource {
                    address: key_pair_address(address),
                    resource_type: "aws_key_pair".into(),
                    name: key_name.clone(),
                    id: Some(key_name.clone()),
                    status: "available".into(),
                    public_ip: None,
                },
            );
        };

        if self
            .ec2()?
            .describe_key_pairs()
            .key_names(&key_name)
            .send()
            .await
            .is_ok()
        {
            let pem = std::path::Path::new(".fleetform")
                .join("keys")
                .join(format!("{}.pem", key_name));
            if !pem.exists() {
                terminal::warn(&format!(
                    "Key pair {} exists in AWS but {} is missing locally. AWS only \
                     hands out the private key once, so SSH will not work until you \
                     delete the key pair and re-apply.",
                    key_name,
                    pem.display()
                ));
            }
            terminal::info(&format!("Reusing key pair {}", key_name));
            record(state);
            persist(state).await?;
            return Ok(key_name);
        }

        let resp = self
            .ec2()?
            .create_key_pair()
            .key_name(&key_name)
            .tag_specifications(
                TagSpecification::builder()
                    .resource_type(ResourceType::KeyPair)
                    .tags(Tag::builder().key("Name").value(&key_name).build())
                    .tags(Tag::builder().key("ManagedBy").value("fleetform").build())
                    .tags(
                        Tag::builder()
                            .key("fleetform:address")
                            .value(address)
                            .build(),
                    )
                    .build(),
            )
            .send()
            .await
            .map_err(|e| anyhow!("CreateKeyPair failed: {}", e))?;

        let material = resp
            .key_material()
            .ok_or_else(|| anyhow!("CreateKeyPair returned no private key material"))?;
        let path = write_private_key(&key_name, material)?;
        terminal::success(&format!(
            "Key pair {} created. Private key: {}",
            key_name,
            path.display()
        ));
        record(state);
        persist(state).await?;
        Ok(key_name)
    }

    /// Create (or adopt) the security group for an instance, allowing SSH from
    /// `ssh_cidr` only.
    async fn ensure_security_group(
        &self,
        address: &str,
        ssh_cidr: &str,
        state: &mut State,
    ) -> Result<String> {
        let group_name = security_group_name(&workspace_name(), address);
        let record = |state: &mut State, id: &str| {
            upsert(
                state,
                ManagedResource {
                    address: security_group_address(address),
                    resource_type: "aws_security_group".into(),
                    name: group_name.clone(),
                    id: Some(id.to_string()),
                    status: "available".into(),
                    public_ip: None,
                },
            );
        };

        let existing = self
            .ec2()?
            .describe_security_groups()
            .filters(
                Filter::builder()
                    .name("group-name")
                    .values(&group_name)
                    .build(),
            )
            .send()
            .await
            .ok()
            .and_then(|r| {
                r.security_groups()
                    .first()
                    .and_then(|g| g.group_id().map(|s| s.to_string()))
            });

        if let Some(id) = existing {
            terminal::info(&format!("Reusing security group {} ({})", group_name, id));
            record(state, &id);
            persist(state).await?;
            return Ok(id);
        }

        let resp = self
            .ec2()?
            .create_security_group()
            .group_name(&group_name)
            .description(format!("fleetform SSH access for {}", address))
            .tag_specifications(
                TagSpecification::builder()
                    .resource_type(ResourceType::SecurityGroup)
                    .tags(Tag::builder().key("Name").value(&group_name).build())
                    .tags(Tag::builder().key("ManagedBy").value("fleetform").build())
                    .tags(
                        Tag::builder()
                            .key("fleetform:address")
                            .value(address)
                            .build(),
                    )
                    .build(),
            )
            .send()
            .await
            .map_err(|e| anyhow!("CreateSecurityGroup failed: {}", e))?;

        let id = resp
            .group_id()
            .ok_or_else(|| anyhow!("CreateSecurityGroup returned no group id"))?
            .to_string();

        self.ec2()?
            .authorize_security_group_ingress()
            .group_id(&id)
            .ip_permissions(
                IpPermission::builder()
                    .ip_protocol("tcp")
                    .from_port(22)
                    .to_port(22)
                    .ip_ranges(
                        IpRange::builder()
                            .cidr_ip(ssh_cidr)
                            .description("fleetform ssh")
                            .build(),
                    )
                    .build(),
            )
            .send()
            .await
            .map_err(|e| anyhow!("AuthorizeSecurityGroupIngress failed: {}", e))?;

        terminal::success(&format!(
            "Security group {} ({}) allows SSH from {}",
            group_name, id, ssh_cidr
        ));
        record(state, &id);
        persist(state).await?;
        Ok(id)
    }

    async fn describe(&self, id: &str) -> Result<Option<ManagedResource>> {
        let resp = match self
            .ec2()?
            .describe_instances()
            .instance_ids(id)
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => return Ok(None),
        };
        for res in resp.reservations() {
            for inst in res.instances() {
                let status = inst
                    .state()
                    .and_then(|s| s.name())
                    .map(|n| n.as_str().to_string())
                    .unwrap_or_else(|| "unknown".into());
                return Ok(Some(ManagedResource {
                    address: String::new(),
                    resource_type: "aws_instance".into(),
                    name: String::new(),
                    id: inst.instance_id().map(|s| s.to_string()),
                    status,
                    public_ip: inst.public_ip_address().map(|s| s.to_string()),
                }));
            }
        }
        Ok(None)
    }

    async fn wait_running(&self, address: &str, name: &str, id: &str) -> Result<ManagedResource> {
        for _ in 0..60 {
            if let Some(live) = self.describe(id).await? {
                if live.status == "running" {
                    return Ok(ManagedResource {
                        address: address.to_string(),
                        resource_type: "aws_instance".into(),
                        name: name.to_string(),
                        id: Some(id.to_string()),
                        status: "running".into(),
                        public_ip: live.public_ip,
                    });
                }
                if live.status == "terminated" || live.status == "shutting-down" {
                    return Err(anyhow!("instance {} entered {}", id, live.status));
                }
                terminal::info(&format!("  {} is {}", id, live.status));
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        Err(anyhow!("timed out waiting for {} to reach running", id))
    }

    pub async fn destroy_all(&self, state: &mut State) -> Result<()> {
        if !live_enabled() {
            let blocking = blocking_cloud_ids(false, state);
            if !blocking.is_empty() {
                return Err(anyhow!(
                    "Refusing to destroy: state records {} live cloud resource(s) [{}]. \
                     Clearing local state would leave them running and billing with no id \
                     left to recover them. Re-run with FLEETFORM_LIVE=1 to terminate them.",
                    blocking.len(),
                    blocking.join(", ")
                ));
            }
            terminal::warn("FLEETFORM_LIVE is not set. Clearing locally planned records only.");
            state.managed.clear();
            state.resources.clear();
            return Ok(());
        }
        let ids = self.recorded_ids(state, "aws_instance");
        let group_ids = self.recorded_ids(state, "aws_security_group");
        let key_names = self.recorded_ids(state, "aws_key_pair");

        if ids.is_empty() && group_ids.is_empty() && key_names.is_empty() {
            terminal::info("No cloud ids in state.");
            state.managed.clear();
            state.resources.clear();
            return Ok(());
        }

        if !ids.is_empty() {
            terminal::warn(&format!("Terminating {:?}", ids));
            self.ec2()?
                .terminate_instances()
                .set_instance_ids(Some(ids.clone()))
                .send()
                .await
                .map_err(|e| anyhow!("TerminateInstances failed: {}", e))?;
            for id in &ids {
                for _ in 0..60 {
                    if let Some(live) = self.describe(id).await? {
                        if live.status == "terminated" {
                            break;
                        }
                        terminal::info(&format!("  {} is {}", id, live.status));
                    }
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }

        for group_id in &group_ids {
            self.delete_security_group(group_id).await?;
        }
        for key_name in &key_names {
            self.ec2()?
                .delete_key_pair()
                .key_name(key_name)
                .send()
                .await
                .map_err(|e| anyhow!("DeleteKeyPair failed for {}: {}", key_name, e))?;
            terminal::success(&format!("Deleted key pair {}", key_name));
        }

        state.managed.clear();
        state.resources.clear();
        Ok(())
    }

    fn recorded_ids(&self, state: &State, resource_type: &str) -> Vec<String> {
        state
            .managed
            .iter()
            .filter(|r| r.resource_type == resource_type)
            .filter_map(|r| r.id.clone())
            .collect()
    }

    /// A security group cannot be deleted while a terminating instance's network
    /// interface still references it, and that reference outlives the
    /// `terminated` state by a few seconds.
    async fn delete_security_group(&self, group_id: &str) -> Result<()> {
        let mut last_err = None;
        for _ in 0..12 {
            match self
                .ec2()?
                .delete_security_group()
                .group_id(group_id)
                .send()
                .await
            {
                Ok(_) => {
                    terminal::success(&format!("Deleted security group {}", group_id));
                    return Ok(());
                }
                Err(e) => {
                    terminal::info(&format!("  {} still in use, retrying", group_id));
                    last_err = Some(e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
        Err(anyhow!(
            "DeleteSecurityGroup failed for {}: {}",
            group_id,
            last_err
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown error".into())
        ))
    }
}

fn upsert(state: &mut State, rec: ManagedResource) {
    if let Some(existing) = state
        .managed
        .iter_mut()
        .find(|r| r.address == rec.address || (rec.id.is_some() && r.id == rec.id))
    {
        *existing = rec.clone();
    } else {
        state.managed.push(rec.clone());
    }
    if !rec.address.is_empty() && !state.resources.iter().any(|r| r == &rec.address) {
        state.resources.push(rec.address);
    }
}

fn client_token(address: &str) -> String {
    let raw = format!("ff-{}-{}", workspace_name(), address.replace('.', "-"));
    raw.chars().take(64).collect()
}

async fn resolve_ami(ec2: &Ec2Client, configured: Option<&str>) -> Result<String> {
    if let Ok(from_env) = std::env::var("FLEETFORM_AMI") {
        if from_env.starts_with("ami-") {
            return Ok(from_env);
        }
    }
    if let Some(ami) = configured {
        if ami.starts_with("ami-") && ami != "ami-12345678" && ami.len() >= 12 {
            return Ok(ami.to_string());
        }
    }
    terminal::info("Resolving current Amazon Linux 2023 AMI");
    let images = ec2
        .describe_images()
        .owners("amazon")
        .filters(
            Filter::builder()
                .name("name")
                .values("al2023-ami-minimal-*-x86_64")
                .build(),
        )
        .filters(Filter::builder().name("state").values("available").build())
        .send()
        .await
        .map_err(|e| anyhow!("DescribeImages failed: {}", e))?;
    let mut imgs: Vec<_> = images.images().iter().collect();
    imgs.sort_by(|a, b| {
        b.creation_date()
            .unwrap_or("")
            .cmp(a.creation_date().unwrap_or(""))
    });
    imgs.first()
        .and_then(|i| i.image_id())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Could not resolve an Amazon Linux AMI. Set FLEETFORM_AMI."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn inst(name: &str) -> Block {
        let mut attributes = HashMap::new();
        attributes.insert("ami".into(), "ami-12345678".into());
        attributes.insert("instance_type".into(), "t3.micro".into());
        Block {
            block_type: "resource".into(),
            labels: vec!["aws_instance".into(), name.into()],
            attributes,
            blocks: vec![],
        }
    }

    #[test]
    fn plan_creates_when_state_empty() {
        let plan = build_plan(&[inst("example")], &State::new());
        assert_eq!(plan.add, 1);
        assert_eq!(plan.changes[0].address, "aws_instance.example");
        assert_eq!(plan.changes[0].action, Action::Create);
    }

    #[test]
    fn plan_is_noop_when_instance_running() {
        let mut state = State::new();
        upsert(
            &mut state,
            ManagedResource {
                address: "aws_instance.example".into(),
                resource_type: "aws_instance".into(),
                name: "example".into(),
                id: Some("i-abc".into()),
                status: "running".into(),
                public_ip: Some("1.2.3.4".into()),
            },
        );
        let plan = build_plan(&[inst("example")], &state);
        assert_eq!(plan.add, 0);
        assert_eq!(plan.changes[0].action, Action::NoOp);
    }

    #[test]
    fn upsert_keeps_hcl_address() {
        let mut state = State::new();
        upsert(
            &mut state,
            ManagedResource {
                address: "aws_instance.example".into(),
                resource_type: "aws_instance".into(),
                name: "example".into(),
                id: Some("i-1".into()),
                status: "running".into(),
                public_ip: None,
            },
        );
        upsert(
            &mut state,
            ManagedResource {
                address: "aws_instance.example".into(),
                resource_type: "aws_instance".into(),
                name: "example".into(),
                id: Some("i-1".into()),
                status: "running".into(),
                public_ip: Some("9.9.9.9".into()),
            },
        );
        assert_eq!(state.managed.len(), 1);
        assert_eq!(state.managed[0].address, "aws_instance.example");
        assert_eq!(state.managed[0].public_ip.as_deref(), Some("9.9.9.9"));
    }

    fn managed(address: &str, id: Option<&str>, status: &str) -> ManagedResource {
        ManagedResource {
            address: address.into(),
            resource_type: "aws_instance".into(),
            name: address.rsplit('.').next().unwrap_or("x").into(),
            id: id.map(|s| s.to_string()),
            status: status.into(),
            public_ip: None,
        }
    }

    #[test]
    fn converged_plan_reports_nothing_to_do() {
        let mut state = State::new();
        upsert(
            &mut state,
            managed("aws_instance.example", Some("i-abc"), "running"),
        );
        let plan = build_plan(&[inst("example")], &state);
        assert_eq!(plan.add, 0);
        assert_eq!(plan.change, 0);
        assert_eq!(plan.destroy, 0);
        assert_eq!(plan.unchanged, 1);
        assert!(
            plan.is_empty(),
            "a stack matching config must report nothing to do"
        );
    }

    #[test]
    fn destroy_without_live_is_blocked_by_recorded_cloud_ids() {
        let mut state = State::new();
        upsert(
            &mut state,
            managed("aws_instance.example", Some("i-abc"), "running"),
        );
        assert_eq!(blocking_cloud_ids(false, &state), vec!["i-abc".to_string()]);
    }

    #[test]
    fn destroy_without_live_allows_planned_only_records() {
        let mut state = State::new();
        upsert(&mut state, managed("aws_instance.example", None, "planned"));
        assert!(blocking_cloud_ids(false, &state).is_empty());
    }

    #[test]
    fn live_destroy_is_never_blocked() {
        let mut state = State::new();
        upsert(
            &mut state,
            managed("aws_instance.example", Some("i-abc"), "running"),
        );
        assert!(blocking_cloud_ids(true, &state).is_empty());
    }

    #[test]
    fn a_pending_instance_still_blocks_a_non_live_destroy() {
        // apply records the id with status "pending" before waiting. If that
        // status escaped the destroy guard, the very record added to prevent
        // orphans would be the one that let state be wiped.
        let mut state = State::new();
        upsert(
            &mut state,
            managed("aws_instance.example", Some("i-abc"), "pending"),
        );
        assert_eq!(blocking_cloud_ids(false, &state), vec!["i-abc".to_string()]);
    }

    #[test]
    fn plan_does_not_promise_a_recreate_it_may_not_perform() {
        let mut state = State::new();
        upsert(
            &mut state,
            managed("aws_instance.example", Some("i-abc"), "pending"),
        );
        let note = &build_plan(&[inst("example")], &state).changes[0].note;
        assert!(
            note.contains("adopts it if it still exists"),
            "plan is offline and cannot know the resource is gone, got {:?}",
            note
        );
        assert!(!note.contains("will recreate"));
    }

    #[test]
    fn resource_names_are_aws_safe_and_stable() {
        let key = key_pair_name("default", "aws_instance.example");
        assert_eq!(key, "fleetform-default-aws-instance-example");
        assert_eq!(key, key_pair_name("default", "aws_instance.example"));
        assert_eq!(
            security_group_name("default", "aws_instance.example"),
            "fleetform-default-aws-instance-example-sg"
        );
        assert!(key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
    }

    #[test]
    fn implied_resources_get_their_own_state_addresses() {
        assert_eq!(key_pair_address("aws_instance.web"), "aws_instance.web#key");
        assert_eq!(
            security_group_address("aws_instance.web"),
            "aws_instance.web#sg"
        );
    }

    #[test]
    fn public_ip_becomes_a_single_host_cidr() {
        assert_eq!(
            cidr_from_public_ip("203.0.113.4\n").unwrap(),
            "203.0.113.4/32"
        );
        assert!(cidr_from_public_ip("not-an-ip").is_err());
        assert!(cidr_from_public_ip("203.0.113").is_err());
        assert!(cidr_from_public_ip("999.0.113.4").is_err());
        assert!(cidr_from_public_ip("").is_err());
    }

    #[tokio::test]
    async fn configured_ssh_cidr_is_used_verbatim_and_must_have_a_prefix() {
        assert_eq!(
            resolve_ssh_cidr(Some("10.0.0.0/8")).await.unwrap(),
            "10.0.0.0/8"
        );
        // No network call happens on this path, so a bad value fails fast.
        assert!(resolve_ssh_cidr(Some("10.0.0.1")).await.is_err());
    }

    #[test]
    fn instance_plan_announces_the_implied_resources() {
        let plan = build_plan(&[inst("example")], &State::new());
        assert!(
            plan.changes[0].note.contains("key pair")
                && plan.changes[0].note.contains("security group"),
            "plan must disclose implied resources, got {:?}",
            plan.changes[0].note
        );
    }

    #[test]
    fn client_token_is_stable_and_bounded() {
        let token = client_token("aws_instance.example");
        assert!(token.starts_with("ff-"));
        assert!(token.len() <= 64);
        assert_eq!(token, client_token("aws_instance.example"));
    }
}
