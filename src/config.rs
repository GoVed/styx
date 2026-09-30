use std::path::PathBuf;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub memory_dir: PathBuf,
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    pub tantivy_dir: PathBuf,
    pub docker_socket: String,
    pub static_dir: PathBuf,
    pub max_concurrent_turns: usize,
    pub allowed_origins: Vec<String>,
    pub docker_host_ip: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let host = std::env::var("STYX_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("STYX_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);
        let memory_dir = std::env::var("STYX_MEMORY_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./memory"));
        let data_dir = std::env::var("STYX_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./data"));
        let db_path = data_dir.join("styx.db");
        let tantivy_dir = data_dir.join("tantivy_index");
        let docker_socket = std::env::var("DOCKER_SOCKET")
            .unwrap_or_else(|_| "/var/run/docker.sock".to_string());
        let static_dir = std::env::var("STYX_STATIC_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./ui/dist"));
        let max_concurrent_turns = std::env::var("STYX_MAX_CONCURRENT_TURNS")
            .ok()
            .and_then(|m| m.parse().ok())
            .unwrap_or(1);
        let allowed_origins = std::env::var("STYX_ALLOWED_ORIGINS")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|item| item.trim().to_string())
                    .filter(|item| !item.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        let docker_host_ip = std::env::var("STYX_DOCKER_HOST_IP")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        Self {
            host,
            port,
            memory_dir,
            data_dir,
            db_path,
            tantivy_dir,
            docker_socket,
            static_dir,
            max_concurrent_turns,
            allowed_origins,
            docker_host_ip,
        }
    }
}
