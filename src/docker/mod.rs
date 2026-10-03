pub mod deploy;
pub mod engines;

use anyhow::{Context, Result};
use bollard::container::{
    ListContainersOptions, LogOutput, LogsOptions, RemoveContainerOptions,
    RestartContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::Docker;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSummaryInfo {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    pub status: String,
    pub state: String,
    pub ports: Vec<String>,
    pub is_syndae_managed: bool,
    pub created: i64,
    pub model_id: Option<String>,
}

#[derive(Clone)]
pub struct DockerOrchestrator {
    docker: Docker,
}

impl DockerOrchestrator {
    pub fn new() -> Result<Self> {
        let docker = Docker::connect_with_socket_defaults()
            .context("Failed to connect to Docker daemon via socket")?;
        Ok(Self { docker })
    }

    #[allow(dead_code)]
    pub fn client(&self) -> &Docker {
        &self.docker
    }

    pub async fn list_containers(&self, all: bool) -> Result<Vec<ContainerSummaryInfo>> {
        let options = ListContainersOptions::<String> {
            all,
            ..Default::default()
        };

        let containers = self.docker.list_containers(Some(options)).await?;
        let mut list = Vec::new();

        for c in containers {
            let id = c.id.unwrap_or_default();
            let names = c.names.unwrap_or_default();
            let image = c.image.unwrap_or_default();
            let status = c.status.unwrap_or_default();
            let state = c.state.unwrap_or_default();
            let created = c.created.unwrap_or_default();

            let is_syndae_managed = names.iter().any(|n| n.contains("syndae"))
                || image.contains("vllm")
                || image.contains("llama.cpp")
                || image.contains("ollama");

            let mut ports_str = Vec::new();
            if let Some(ports) = c.ports {
                for p in ports {
                    let pub_port = p.public_port.map(|v| v.to_string()).unwrap_or_default();
                    let priv_port = p.private_port;
                    let p_type = p.typ.map(|t| format!("{:?}", t)).unwrap_or_default();
                    if !pub_port.is_empty() {
                        ports_str.push(format!("{}:{} ({})", pub_port, priv_port, p_type));
                    } else {
                        ports_str.push(format!("{} ({})", priv_port, p_type));
                    }
                }
            }

            let mut detected_model = None;
            if let Some(cmd) = &c.command {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if image.contains("vllm") || cmd.contains("vllm") {
                    if let Some(pos) = parts.iter().position(|&p| p == "serve") {
                        if let Some(m) = parts.get(pos + 1)
                            && !m.starts_with('-') {
                                detected_model = Some(m.to_string());
                            }
                    } else if let Some(first) = parts.first()
                        && !first.starts_with('-') && *first != "vllm" {
                            detected_model = Some(first.to_string());
                        }
                } else if (image.contains("llama.cpp") || cmd.contains("llama"))
                    && let Some(pos) = parts.iter().position(|&p| p == "-hf" || p == "-m")
                        && let Some(m) = parts.get(pos + 1) {
                            detected_model = Some(m.to_string());
                        }
            }

            list.push(ContainerSummaryInfo {
                id,
                names,
                image,
                status,
                state,
                ports: ports_str,
                is_syndae_managed,
                created,
                model_id: detected_model,
            });
        }

        Ok(list)
    }

    pub async fn pull_image_if_missing(&self, image: &str) -> Result<()> {
        if self.docker.inspect_image(image).await.is_ok() {
            info!("Image already present locally: {}", image);
            return Ok(());
        }

        info!("Image not found locally, pulling: {}", image);
        let options = CreateImageOptions {
            from_image: image,
            ..Default::default()
        };

        let mut stream = self.docker.create_image(Some(options), None, None);
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(item) => {
                    if let Some(status) = item.status {
                        tracing::debug!("Image pull status: {}", status);
                    }
                }
                Err(e) => {
                    tracing::warn!("Pull warning: {:?}", e);
                }
            }
        }
        Ok(())
    }

    pub async fn start_container(&self, id: &str) -> Result<()> {
        self.docker
            .start_container(id, None::<StartContainerOptions<String>>)
            .await?;
        Ok(())
    }

    pub async fn stop_container(&self, id: &str) -> Result<()> {
        let options = StopContainerOptions { t: 10 };
        self.docker.stop_container(id, Some(options)).await?;
        Ok(())
    }

    pub async fn restart_container(&self, id: &str) -> Result<()> {
        let options = RestartContainerOptions { t: 10 };
        self.docker.restart_container(id, Some(options)).await?;
        Ok(())
    }

    pub async fn delete_container(&self, id: &str) -> Result<()> {
        let options = RemoveContainerOptions {
            force: true,
            ..Default::default()
        };
        self.docker.remove_container(id, Some(options)).await?;
        Ok(())
    }

    pub async fn inspect_container(&self, id: &str) -> Result<bollard::models::ContainerInspectResponse> {
        let inspect = self.docker.inspect_container(id, None).await?;
        Ok(inspect)
    }

    pub async fn get_container_logs(&self, id: &str, tail: usize) -> Result<Vec<String>> {
        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: tail.to_string(),
            timestamps: true,
            ..Default::default()
        };

        let mut stream = self.docker.logs(id, Some(options));
        let mut lines = Vec::new();

        while let Some(msg) = stream.next().await {
            match msg {
                Ok(LogOutput::StdOut { message }) => {
                    lines.push(String::from_utf8_lossy(&message).trim().to_string());
                }
                Ok(LogOutput::StdErr { message }) => {
                    lines.push(String::from_utf8_lossy(&message).trim().to_string());
                }
                Ok(LogOutput::Console { message }) => {
                    lines.push(String::from_utf8_lossy(&message).trim().to_string());
                }
                Ok(_) => {}
                Err(e) => {
                    error!("Error reading logs: {:?}", e);
                    break;
                }
            }
        }

        Ok(lines)
    }
}
