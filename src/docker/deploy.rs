use std::collections::HashMap;
use anyhow::Result;
use bollard::container::{Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions};
use bollard::models::{DeviceMapping, DeviceRequest, HostConfig, PortBinding};
use tracing::info;

use super::engines::{self, DeployModelRequest};
use super::DockerOrchestrator;

impl DockerOrchestrator {
    pub async fn deploy_model_container(&self, req: &DeployModelRequest) -> Result<String> {
        let vendor = req.resolved_gpu_vendor();
        let image = req.image_name();
        let _ = self.pull_image_if_missing(&image).await;

        let host_port = req.default_port();
        let target_port = match req.engine {
            engines::EngineKind::Vllm => 8000,
            engines::EngineKind::LlamaCpp => 8080,
            engines::EngineKind::Ollama => 11434,
        };

        let bind_ip = std::env::var("STYX_DOCKER_HOST_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port_key = format!("{}/tcp", target_port);
        let mut port_bindings = HashMap::new();
        port_bindings.insert(
            port_key.clone(),
            Some(vec![PortBinding {
                host_ip: Some(bind_ip),
                host_port: Some(host_port.to_string()),
            }]),
        );

        let mut exposed_ports = HashMap::new();
        exposed_ports.insert(port_key, HashMap::new());

        // Setup GPU device requests and device mappings based on resolved GPU vendor
        let (device_requests, devices, group_add, security_opt) = match vendor {
            engines::GpuVendor::Nvidia => {
                let dev_reqs = if let Some(gpu) = &req.gpu_devices {
                    if !gpu.is_empty() {
                        let count = if gpu == "all" { -1 } else { 1 };
                        let device_ids = if gpu == "all" {
                            None
                        } else {
                            Some(gpu.split(',').map(|s| s.trim().to_string()).collect())
                        };

                        Some(vec![DeviceRequest {
                            driver: Some("nvidia".to_string()),
                            count: Some(count),
                            device_ids,
                            capabilities: Some(vec![vec!["gpu".to_string(), "compute".to_string()]]),
                            options: None,
                        }])
                    } else {
                        None
                    }
                } else {
                    None
                };
                (dev_reqs, None, None, None)
            }
            engines::GpuVendor::Amd => {
                let devices = Some(vec![
                    DeviceMapping {
                        path_on_host: Some("/dev/kfd".to_string()),
                        path_in_container: Some("/dev/kfd".to_string()),
                        cgroup_permissions: Some("rwm".to_string()),
                    },
                    DeviceMapping {
                        path_on_host: Some("/dev/dri".to_string()),
                        path_in_container: Some("/dev/dri".to_string()),
                        cgroup_permissions: Some("rwm".to_string()),
                    },
                ]);
                let group_add = Some(vec!["video".to_string(), "render".to_string()]);
                let security_opt = Some(vec!["seccomp=unconfined".to_string(), "label=disable".to_string()]);
                (None, devices, group_add, security_opt)
            }
            _ => (None, None, None, None),
        };

        let host_home = std::env::var("HOST_HOME")
            .unwrap_or_else(|_| std::env::var("HOME").unwrap_or_else(|_| "/root".to_string()));
        let hf_cache = format!("{}/.cache/huggingface:/root/.cache/huggingface:rw", host_home);
        let mut binds = vec![hf_cache];

        let host_local_models = std::env::var("HOST_LOCAL_MODELS_DIR")
            .unwrap_or_else(|_| format!("{}/localLLM/models", host_home));
        binds.push(format!("{}:/models:ro", host_local_models));

        let host_config = HostConfig {
            port_bindings: Some(port_bindings),
            device_requests,
            devices,
            group_add,
            security_opt,
            binds: Some(binds),
            ipc_mode: Some("host".to_string()),
            ..Default::default()
        };

        let env_vars = req.build_env();
        let cmd_args = req.build_cmd_args();

        let mut labels = HashMap::new();
        labels.insert("styx.managed".to_string(), "true".to_string());
        labels.insert("styx.engine".to_string(), req.engine.to_string());
        labels.insert("styx.model".to_string(), req.hf_repo.clone());

        let container_name = format!("styx-{}", req.name.trim().replace(' ', "-").to_lowercase());

        let config = Config {
            image: Some(image.to_string()),
            cmd: if cmd_args.is_empty() {
                None
            } else {
                Some(cmd_args)
            },
            env: if env_vars.is_empty() {
                None
            } else {
                Some(env_vars)
            },
            exposed_ports: Some(exposed_ports),
            host_config: Some(host_config),
            labels: Some(labels),
            ..Default::default()
        };

        // Force-remove existing container with the same name or port collision if any
        let rm_options = RemoveContainerOptions {
            force: true,
            ..Default::default()
        };
        let _ = self.docker.remove_container(&container_name, Some(rm_options)).await;

        if let Ok(containers) = self.list_containers(true).await {
            let port_needle = format!(":{}", host_port);
            for c in containers {
                if (c.is_styx_managed || c.names.iter().any(|n| n.starts_with("/styx-")))
                    && c.ports.iter().any(|p| p.contains(&port_needle)) {
                        let _ = self.docker.remove_container(&c.id, Some(RemoveContainerOptions { force: true, ..Default::default() })).await;
                    }
            }
        }

        let options = CreateContainerOptions {
            name: container_name.clone(),
            platform: None,
        };

        info!("Creating container: {}", container_name);
        let res = self.docker.create_container(Some(options), config).await?;
        let container_id = res.id;

        info!("Starting container: {}", container_id);
        self.docker
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await?;

        Ok(container_id)
    }
}
