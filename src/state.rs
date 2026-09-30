use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use fs4::FileExt;

use aws_sdk_s3::{primitives::ByteStream, Client as S3Client};
use serde::{Deserialize, Serialize};

fn with_retry<T, F>(operation: F, retries: u32) -> Result<T, anyhow::Error>
where
    F: Fn() -> Result<T, anyhow::Error>,
{
    for _ in 0..retries {
        match operation() {
            Ok(result) => return Ok(result),
            Err(_) => thread::sleep(Duration::from_secs(1)),
        }
    }
    operation()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManagedResource {
    pub address: String,
    pub resource_type: String,
    pub name: String,
    pub id: Option<String>,
    pub status: String,
    pub public_ip: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct State {
    version: u32,
    serial: u64,
    pub resources: Vec<String>,
    #[serde(default)]
    pub managed: Vec<ManagedResource>,
}

impl State {
    pub fn new() -> Self {
        State {
            version: 1,
            serial: 0,
            resources: vec![],
            managed: vec![],
        }
    }

    pub fn cleanup_backup(path: &Path) -> Result<(), anyhow::Error> {
        let backup_path = path.with_extension("backup");
        if backup_path.exists() {
            fs::remove_file(&backup_path)?;
        }
        Ok(())
    }

    pub fn read(path: &Path) -> Result<Self> {
        with_retry(
            || {
                let mut file = File::open(path)?;
                FileExt::lock_shared(&file)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                FileExt::unlock(&file)?;
                Ok(serde_json::from_str(&contents)?)
            },
            3,
        )
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        Self::cleanup_backup(path)?;
        with_retry(
            || {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut file = File::create(path)?;
                FileExt::lock_exclusive(&file)?;
                let contents = serde_json::to_string_pretty(self)?;
                file.write_all(contents.as_bytes())?;
                file.flush()?;
                FileExt::unlock(&file)?;
                Ok(())
            },
            3,
        )
    }

    pub async fn write_s3(&self, bucket: &str, key: &str) -> Result<()> {
        let contents = serde_json::to_string(self)?;
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .load()
            .await;
        let client = S3Client::new(&config);

        client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(ByteStream::from(contents.into_bytes()))
            .send()
            .await?;

        Ok(())
    }

    pub async fn write_consul(&self, endpoint: &str, key: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let contents = serde_json::to_string(self)?;
        let url = format!("{}/v1/kv/{}", endpoint, key);
        let response = client.put(&url).body(contents).send().await?;
        if response.status().is_success() {
            crate::terminal::success(&format!("State written to Consul at {}: {}", endpoint, key));
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to write to Consul: {}",
                response.status()
            ))
        }
    }

    pub async fn read_consul(endpoint: &str, key: &str) -> Result<Self> {
        let client = reqwest::Client::new();
        let url = format!("{}/v1/kv/{}?raw", endpoint, key);
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let contents = response.text().await?;
                    Ok(serde_json::from_str(&contents)?)
                } else if response.status() == 404 {
                    Ok(State::new())
                } else {
                    Err(anyhow::anyhow!(
                        "Failed to read from Consul: {}",
                        response.status()
                    ))
                }
            }
            Err(e) => {
                crate::terminal::warn(&format!("Consul connection failed: {}", e));
                Ok(State::new())
            }
        }
    }
}

pub async fn load() -> Result<State> {
    let path = Path::new(".fleetform/state.json");
    if path.exists() {
        State::read(path)
    } else {
        Ok(State::new())
    }
}

pub async fn save(state: &State) -> Result<()> {
    std::fs::create_dir_all(".fleetform")?;
    let path = Path::new(".fleetform/state.json");
    state.write(path)
}
