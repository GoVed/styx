use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Stdio;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tracing::{debug, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: &'static str,
    pub id: i64,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<Value>,
}

pub enum TransportHandle {
    Stdio {
        _child: Child,
        stdin: Arc<Mutex<ChildStdin>>,
        reader: Arc<Mutex<BufReader<ChildStdout>>>,
    },
    UnixSocket {
        stream: Arc<Mutex<UnixStream>>,
    },
    Http {
        client: reqwest::Client,
        url: String,
    },
}

pub struct McpTransport {
    handle: TransportHandle,
    next_id: AtomicI64,
}

impl McpTransport {
    pub async fn connect_stdio(
        command: &str,
        args: &[String],
        env: &[(String, String)],
    ) -> Result<Self> {
        info!("Spawning stdio MCP daemon: {} {:?}", command, args);

        let mut cmd = Command::new(command);
        cmd.args(args);
        for (k, v) in env {
            cmd.env(k, v);
        }
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());

        let mut child = cmd.spawn().with_context(|| {
            format!("Failed to spawn MCP subprocess: {} {:?}", command, args)
        })?;

        let stdin = child
            .stdin
            .take()
            .context("Failed to capture child stdin")?;
        let stdout = child
            .stdout
            .take()
            .context("Failed to capture child stdout")?;

        let transport = Self {
            handle: TransportHandle::Stdio {
                _child: child,
                stdin: Arc::new(Mutex::new(stdin)),
                reader: Arc::new(Mutex::new(BufReader::new(stdout))),
            },
            next_id: AtomicI64::new(1),
        };

        Ok(transport)
    }

    pub async fn connect_unix(socket_path: &str) -> Result<Self> {
        info!("Connecting to Unix socket MCP daemon: {}", socket_path);
        let stream = UnixStream::connect(socket_path)
            .await
            .with_context(|| format!("Failed to connect to unix socket at {}", socket_path))?;

        Ok(Self {
            handle: TransportHandle::UnixSocket {
                stream: Arc::new(Mutex::new(stream)),
            },
            next_id: AtomicI64::new(1),
        })
    }

    pub fn connect_http(url: &str) -> Self {
        info!("Setting up HTTP MCP client for: {}", url);
        Self {
            handle: TransportHandle::Http {
                client: reqwest::Client::new(),
                url: url.to_string(),
            },
            next_id: AtomicI64::new(1),
        }
    }

    pub async fn call_method(&self, method: &str, params: Value) -> Result<Value> {
        let req_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: req_id,
            method: method.to_string(),
            params,
        };

        let req_bytes = serde_json::to_vec(&req)?;

        match &self.handle {
            TransportHandle::Stdio { stdin, reader, .. } => {
                {
                    let mut sin = stdin.lock().await;
                    sin.write_all(&req_bytes).await?;
                    sin.write_all(b"\n").await?;
                    sin.flush().await?;
                }

                let mut r = reader.lock().await;
                let mut line = String::new();
                loop {
                    line.clear();
                    let n = r.read_line(&mut line).await?;
                    if n == 0 {
                        bail!("MCP process closed connection unexpectedly");
                    }
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Try parsing as JSON-RPC response
                    if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(trimmed) {
                        if let Some(err) = resp.error {
                            bail!("MCP error (code {}): {}", err.code, err.message);
                        }
                        return Ok(resp.result.unwrap_or(Value::Null));
                    }
                    debug!("Non-JSON line from MCP daemon: {}", trimmed);
                }
            }
            TransportHandle::UnixSocket { stream } => {
                let mut s = stream.lock().await;
                s.write_all(&req_bytes).await?;
                s.write_all(b"\n").await?;
                s.flush().await?;

                let mut buf_reader = BufReader::new(&mut *s);
                let mut line = String::new();
                let n = buf_reader.read_line(&mut line).await?;
                if n == 0 {
                    bail!("Unix socket connection closed by peer");
                }
                let resp: JsonRpcResponse = serde_json::from_str(line.trim())
                    .context("Invalid JSON-RPC response from unix socket")?;
                if let Some(err) = resp.error {
                    bail!("MCP error (code {}): {}", err.code, err.message);
                }
                Ok(resp.result.unwrap_or(Value::Null))
            }
            TransportHandle::Http { client, url } => {
                let res = client
                    .post(url)
                    .json(&req)
                    .send()
                    .await
                    .context("HTTP request to MCP server failed")?;
                let resp: JsonRpcResponse = res.json().await?;
                if let Some(err) = resp.error {
                    bail!("MCP error (code {}): {}", err.code, err.message);
                }
                Ok(resp.result.unwrap_or(Value::Null))
            }
        }
    }
}
