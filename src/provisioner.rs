use std::collections::HashMap;
use crate::{config::Config, state::State, terminal, hcl::parse_blocks};
use aws_sdk_s3::Client as S3Client;
use aws_sdk_ec2::Client as Ec2Client;



pub struct Provisioner {
    s3_client: S3Client,
    ec2_client: Ec2Client,
}

impl Provisioner {
    pub async fn new() -> Self {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .load()
            .await;
        
        Self {
            s3_client: S3Client::new(&config),
            ec2_client: Ec2Client::new(&config),
        }
    }

    /// Lists EC2 instances with a specific environment tag
    /// 
    /// # Arguments
    /// * `environment` - The environment tag value to filter by (e.g., "Dev", "Prod")
    ///
    /// # Returns
    /// * `Result<Vec<(Option<String>, Option<String>)>>` - A vector of tuples containing
    ///   (instance_id, public_ip) pairs for matching instances
    async fn describe_tagged_instances(&self, environment: &str) -> anyhow::Result<Vec<(Option<String>, Option<String>)>> {
        // Create the filter for running instances with the specified environment tag
        let filters = vec![
            aws_sdk_ec2::types::Filter::builder()
                .name("instance-state-name")
                .values("running")
                .build(),
            aws_sdk_ec2::types::Filter::builder()
                .name("tag:Environment")
                .values(environment)
                .build(),
        ];

        // Describe instances with the specified filters
        let response = self.ec2_client
            .describe_instances()
            .set_filters(Some(filters))
            .send()
            .await?;

        let mut instances = Vec::new();

        // Process the response and extract instance information
        for reservation in response.reservations() {
            for instance in reservation.instances() {
                let instance_id = instance.instance_id().map(String::from);
                let public_ip = instance.public_ip_address().map(String::from);
                instances.push((instance_id, public_ip));
            }
        }

        Ok(instances)
    }
    
    pub async fn provision_resources(&self, _config: &Config, state: &mut State) -> anyhow::Result<()> {
        // First, describe instances to check what's already running
        let instances = self.describe_tagged_instances("Dev").await?;
        for instance in instances {
            if let (Some(id), Some(ip)) = instance {
                terminal::info(&format!("Found running instance {} with IP {}", id, ip));
            }
        }

        // Then process the Terraform configuration
        let contents = std::fs::read_to_string("main.tf")?;
        let blocks = parse_blocks(&contents)?;
        
        for block in blocks {
            if block.block_type != "resource" {
                continue;
            }
            
            if block.labels.len() != 2 {
                return Err(anyhow::anyhow!("Invalid resource block: expected resource type and name"));
            }
            
            let resource_type = &block.labels[0];
            let resource_name = &block.labels[1];
            let attributes = block.attributes;
            
            match resource_type.as_str() {
                "aws_instance" => self.create_ec2_instance(resource_name, &attributes).await?,
                "aws_s3_bucket" => self.create_s3_bucket(resource_name, &attributes).await?,
                _ => terminal::warn(&format!("Unknown resource type: {}", resource_type)),
            }
            
            state.resources.push(format!("{}:{}", resource_type, resource_name));
        }
        
        Ok(())
    }

    pub async fn create_ec2_instance(&self, name: &str, attributes: &HashMap<String, String>) -> anyhow::Result<()> {
        terminal::info(&format!("Creating EC2 instance: {}", name));
        
        let instance_type = attributes.get("instance_type")
            .map(|s| s.as_str())
            .unwrap_or("t2.micro");
        
        let ami = attributes.get("ami")
            .map(|s| s.as_str())
            .unwrap_or("ami-12345678");
            
        let result = self.ec2_client.run_instances()
            .image_id(ami)
            .instance_type(instance_type.into())
            .min_count(1)
            .max_count(1)
            .send()
            .await?;

        if let Some(instance) = result.instances().first() {
            if let Some(id) = instance.instance_id() {
                terminal::success(&format!("EC2 instance created: {}", id));
            }
        }
        
        Ok(())
    }
    
    pub async fn create_s3_bucket(&self, name: &str, attributes: &HashMap<String, String>) -> anyhow::Result<()> {
        terminal::info(&format!("Creating S3 bucket: {}", name));
        
        let bucket_name = attributes.get("bucket")
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("fleetform-bucket-{}", uuid::Uuid::new_v4().simple()));
        
        match self.s3_client.create_bucket()
            .bucket(&bucket_name)
            .send()
            .await {
                Ok(_) => {
                    terminal::success(&format!("S3 bucket created: {}", bucket_name));
                }
                Err(e) => {
                    terminal::error(&format!("Failed to create S3 bucket: {}", e));
                    return Err(anyhow::anyhow!("Failed to create S3 bucket: {}", e));
                }
            }
        Ok(())
    }

}
