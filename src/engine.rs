use anyhow::{anyhow, Result};
use aws_sdk_ec2::types::{Filter, InstanceStateName, Tag, TagSpecification, ResourceType};
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

pub fn load_desired() -> Result<Vec<Block>> {
    let path = if std::path::Path::new("main.tf").exists() {
        "main.tf"
    } else {
        return Err(anyhow!("No main.tf in the working directory"));
    };
    let contents = std::fs::read_to_string(path)?;
    crate::hcl::validate_hcl_syntax(&contents)?;
    parse_blocks(&contents)
}

pub fn build_plan(desired: &[Block], state: &State) -> Plan {
    let live = live_enabled();
    let mut changes = Vec::new();
    for block in desired {
        if block.block_type != "resource" || block.labels.len() != 2 {
            continue;
        }
        let resource_type = block.labels[0].clone();
        let name = block.labels[1].clone();
        let address = format!("{}.{}", resource_type, name);
        let existing = state.managed.iter().find(|r| r.address == address);
        let (action, note) = match existing {
            Some(r) if r.status == "running" || r.status == "available" => {
                (Action::NoOp, format!("already {}", r.status))
            }
            Some(r) if r.id.is_some() => (
                Action::Create,
                format!("prior id {} is not running; will recreate if live", r.id.clone().unwrap_or_default()),
            ),
            _ => (Action::Create, "will be created".to_string()),
        };
        changes.push(PlannedChange {
            address,
            resource_type: resource_type.clone(),
            name,
            action,
            ami: block.attributes.get("ami").cloned(),
            instance_type: block.attributes.get("instance_type").cloned(),
            bucket: block.attributes.get("bucket").cloned(),
            note,
        });
    }
    let add = changes.iter().filter(|c| c.action == Action::Create).count();
    Plan {
        changes,
        add,
        change: 0,
        destroy: 0,
        live,
    }
}

pub struct Engine {
    ec2: Ec2Client,
}

impl Engine {
    pub async fn new() -> Self {
        let cfg = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .load()
            .await;
        Self {
            ec2: Ec2Client::new(&cfg),
        }
    }

    pub async fn apply_plan(&self, plan: &Plan, desired: &[Block], state: &mut State) -> Result<()> {
        if !plan.live {
            terminal::warn("FLEETFORM_LIVE is not set. Plan recorded; no cloud writes.");
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
            let block = desired.iter().find(|b| {
                b.block_type == "resource"
                    && b.labels.len() == 2
                    && format!("{}.{}", b.labels[0], b.labels[1]) == change.address
            });
            let Some(block) = block else {
                continue;
            };
            match change.resource_type.as_str() {
                "aws_instance" => self.apply_instance(block, state).await?,
                "aws_s3_bucket" => {
                    terminal::warn(&format!(
                        "S3 create for {} skipped in Phase 1. EC2 is the launch path.",
                        change.address
                    ));
                }
                other => terminal::warn(&format!("unsupported resource type: {}", other)),
            }
        }
        Ok(())
    }

    async fn apply_instance(&self, block: &Block, state: &mut State) -> Result<()> {
        let address = format!("{}.{}", block.labels[0], block.labels[1]);
        if let Some(existing) = state.managed.iter().find(|r| r.address == address && r.id.is_some()) {
            if let Some(id) = &existing.id {
                if let Some(live) = self.describe(id).await? {
                    if live.status == "running" {
                        terminal::success(&format!(
                            "{} already running as {} {}",
                            address,
                            id,
                            live.public_ip.clone().unwrap_or_default()
                        ));
                        upsert(state, live);
                        return Ok(());
                    }
                }
            }
        }

        let instance_type = block
            .attributes
            .get("instance_type")
            .cloned()
            .unwrap_or_else(|| "t3.micro".into());
        let ami = resolve_ami(&self.ec2, block.attributes.get("ami").map(|s| s.as_str())).await?;

        terminal::info(&format!(
            "Launching {} ami={} type={}",
            address, ami, instance_type
        ));

        let tags = TagSpecification::builder()
            .resource_type(ResourceType::Instance)
            .tags(Tag::builder().key("Name").value(format!("fleetform-{}", block.labels[1])).build())
            .tags(Tag::builder().key("ManagedBy").value("fleetform").build())
            .build();

        let result = self
            .ec2
            .run_instances()
            .image_id(&ami)
            .instance_type(instance_type.as_str().into())
            .min_count(1)
            .max_count(1)
            .client_token(&format!("fleetform-{}", block.labels[1]))
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

        terminal::info(&format!("Waiting for {} to reach running...", id));
        let live = self.wait_running(&id).await?;
        terminal::success(&format!(
            "Machine running: {} id={} ip={}",
            address,
            id,
            live.public_ip.clone().unwrap_or_else(|| "none".into())
        ));
        upsert(state, live);
        Ok(())
    }

    async fn describe(&self, id: &str) -> Result<Option<ManagedResource>> {
        let resp = self.ec2.describe_instances().instance_ids(id).send().await;
        let Ok(resp) = resp else {
            return Ok(None);
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

    async fn wait_running(&self, id: &str) -> Result<ManagedResource> {
        for _ in 0..60 {
            if let Some(mut live) = self.describe(id).await? {
                if live.status == InstanceStateName::Running.as_str() {
                    live.address = format!("aws_instance.{}", live.name);
                    live.resource_type = "aws_instance".into();
                    return Ok(ManagedResource {
                        address: format!("aws_instance.{}", id),
                        resource_type: "aws_instance".into(),
                        name: id.to_string(),
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
            terminal::warn("FLEETFORM_LIVE is not set. Destroy will only clear local state.");
            state.managed.clear();
            state.resources.clear();
            return Ok(());
        }
        let ids: Vec<String> = state
            .managed
            .iter()
            .filter(|r| r.resource_type == "aws_instance")
            .filter_map(|r| r.id.clone())
            .collect();
        if ids.is_empty() {
            terminal::info("No instance ids in state.");
            state.managed.clear();
            state.resources.clear();
            return Ok(());
        }
        terminal::warn(&format!("Terminating {:?}", ids));
        self.ec2
            .terminate_instances()
            .set_instance_ids(Some(ids.clone()))
            .send()
            .await
            .map_err(|e| anyhow!("TerminateInstances failed: {}", e))?;
        for id in ids {
            for _ in 0..60 {
                if let Some(live) = self.describe(&id).await? {
                    if live.status == "terminated" {
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
        state.managed.clear();
        state.resources.clear();
        Ok(())
    }
}

fn upsert(state: &mut State, mut rec: ManagedResource) {
    if rec.address.is_empty() {
        if let Some(id) = &rec.id {
            rec.address = format!("aws_instance.{}", id);
        }
    }
    if let Some(existing) = state.managed.iter_mut().find(|r| r.address == rec.address || r.id == rec.id) {
        *existing = rec.clone();
    } else {
        state.managed.push(rec.clone());
    }
    let label = rec.address.clone();
    if !state.resources.iter().any(|r| r == &label) {
        state.resources.push(label);
    }
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
    imgs.sort_by(|a, b| b.creation_date().unwrap_or("").cmp(a.creation_date().unwrap_or("")));
    imgs.first()
        .and_then(|i| i.image_id())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Could not resolve an Amazon Linux AMI. Set FLEETFORM_AMI."))
}
