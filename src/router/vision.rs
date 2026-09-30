use anyhow::{bail, Context, Result};
use base64::Engine;
use image::ImageFormat;
use reqwest::Client;
use serde_json::{json, Value};
use std::io::Cursor;
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info, warn};

use super::types::ChatMessageParam;

pub fn adapt_docker_url(url: &str) -> String {
    let mut clean = url.trim_end_matches('/').to_string();
    if Path::new("/.dockerenv").exists() {
        if clean.starts_with("http://localhost:") {
            clean = clean.replace("http://localhost:", "http://host.docker.internal:");
        } else if clean.starts_with("http://127.0.0.1:") {
            clean = clean.replace("http://127.0.0.1:", "http://host.docker.internal:");
        }
    }
    clean
}

pub async fn load_image_bytes(input: &str) -> Result<Vec<u8>> {
    let trimmed = input.trim();
    if trimmed.starts_with("data:image/")
        && let Some(pos) = trimmed.find("base64,") {
            let b64_part = &trimmed[pos + 7..];
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(b64_part.trim())
                .context("Failed to decode base64 data URI")?;
            return Ok(decoded);
        }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        let fetch_url = adapt_docker_url(trimmed);
        debug!("Fetching remote image from {}", fetch_url);
        let client = Client::builder()
            .timeout(Duration::from_secs(12))
            .build()
            .unwrap_or_default();

        let resp = client
            .get(&fetch_url)
            .header("User-Agent", "StyxAgentOS/1.0 (VisionRouter)")
            .send()
            .await
            .context(format!("Failed to reach image URL: {}", fetch_url))?;

        if !resp.status().is_success() {
            bail!("Image download failed with status {}: {}", resp.status(), fetch_url);
        }

        let bytes = resp.bytes().await.context("Failed to read image response bytes")?;
        return Ok(bytes.to_vec());
    }

    let local_path = Path::new(trimmed);
    if local_path.exists() {
        let bytes = tokio::fs::read(local_path)
            .await
            .context(format!("Failed to read local image file at {}", trimmed))?;
        return Ok(bytes);
    }

    bail!("Could not resolve image source (not a valid data URI, HTTP URL, or local file): {}", trimmed)
}

pub fn convert_and_downscale_to_jpeg_base64(bytes: &[u8], max_dim: u32) -> Result<String> {
    let img = image::load_from_memory(bytes).context("Failed to decode image format")?;
    let (w, h) = (img.width(), img.height());

    let final_img = if w > max_dim || h > max_dim {
        img.thumbnail(max_dim, max_dim)
    } else {
        img
    };

    let mut jpeg_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut jpeg_bytes);
    final_img
        .write_to(&mut cursor, ImageFormat::Jpeg)
        .context("Failed to encode image to JPEG")?;

    let encoded = base64::engine::general_purpose::STANDARD.encode(&jpeg_bytes);
    Ok(format!("data:image/jpeg;base64,{}", encoded))
}

pub async fn resolve_image_to_jpeg_data_uri(input: &str) -> Result<String> {
    let bytes = load_image_bytes(input).await?;
    convert_and_downscale_to_jpeg_base64(&bytes, 512)
}

pub async fn prepare_openai_messages(messages: &[ChatMessageParam]) -> Vec<Value> {
    let mut out = Vec::with_capacity(messages.len());

    for msg in messages {
        let has_images = msg.images.as_ref().map(|imgs| !imgs.is_empty()).unwrap_or(false);

        if !has_images {
            let mut val = json!({
                "role": msg.role,
                "content": msg.content,
            });
            if let Some(ref name) = msg.name {
                val["name"] = json!(name);
            }
            if let Some(ref tcid) = msg.tool_call_id {
                val["tool_call_id"] = json!(tcid);
            }
            if let Some(ref tcalls) = msg.tool_calls {
                val["tool_calls"] = json!(tcalls);
            }
            out.push(val);
            continue;
        }

        let mut content_parts: Vec<Value> = Vec::new();
        if !msg.content.is_empty() {
            content_parts.push(json!({
                "type": "text",
                "text": msg.content
            }));
        }

        if let Some(ref imgs) = msg.images {
            for img_src in imgs {
                match resolve_image_to_jpeg_data_uri(img_src).await {
                    Ok(data_uri) => {
                        content_parts.push(json!({
                            "type": "image_url",
                            "image_url": {
                                "url": data_uri
                            }
                        }));
                    }
                    Err(e) => {
                        warn!("Failed to resolve image '{}' for multimodal prompt: {}", img_src, e);
                    }
                }
            }
        }

        let mut val = json!({
            "role": msg.role,
            "content": content_parts,
        });
        if let Some(ref name) = msg.name {
            val["name"] = json!(name);
        }
        if let Some(ref tcid) = msg.tool_call_id {
            val["tool_call_id"] = json!(tcid);
        }
        if let Some(ref tcalls) = msg.tool_calls {
            val["tool_calls"] = json!(tcalls);
        }
        out.push(val);
    }

    out
}

pub async fn inspect_image_with_model(
    base_url: &str,
    api_key: &str,
    model: &str,
    image_src: &str,
    question: &str,
) -> Result<String> {
    let clean_base = adapt_docker_url(base_url);
    let chat_url = format!("{}/chat/completions", clean_base);

    info!("Inspecting image via vision model at {} (source: {})", chat_url, image_src);
    let data_uri = resolve_image_to_jpeg_data_uri(image_src).await?;

    let prompt_text = if question.trim().is_empty() {
        "Describe what is shown in this image in detail, including key subjects, expressions, colors, and any text."
    } else {
        question.trim()
    };

    let payload = json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": [
                    { "type": "text", "text": prompt_text },
                    { "type": "image_url", "image_url": { "url": data_uri } }
                ]
            }
        ],
        "max_tokens": 1024,
        "temperature": 0.2,
        "stream": false
    });

    let client = Client::builder()
        .timeout(Duration::from_secs(45))
        .build()
        .unwrap_or_default();

    let mut req = client.post(&chat_url).json(&payload);
    if !api_key.is_empty() {
        req = req.bearer_auth(api_key);
    }

    let resp = req
        .send()
        .await
        .context(format!("Failed to connect to vision model endpoint at {}", chat_url))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err_body = resp.text().await.unwrap_or_default();
        bail!("Vision model API error HTTP {}: {}", status, err_body);
    }

    let result_json: Value = resp.json().await.context("Failed to parse vision model response")?;
    let message_obj = result_json
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c0| c0.get("message"));

    let content = message_obj
        .and_then(|m| m.get("content"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .trim();

    if !content.is_empty() {
        return Ok(content.to_string());
    }

    let reasoning = message_obj
        .and_then(|m| m.get("reasoning_content"))
        .and_then(|r| r.as_str())
        .unwrap_or("")
        .trim();

    if !reasoning.is_empty() {
        return Ok(reasoning.to_string());
    }

    bail!("Vision model returned an empty description for the image");
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    #[tokio::test]
    async fn test_convert_and_downscale_to_jpeg_base64() {
        let mut img = RgbImage::new(100, 100);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 0, 0]); // Pure red
        }
        let mut png_bytes = Vec::new();
        img.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
            .unwrap();

        let data_uri = convert_and_downscale_to_jpeg_base64(&png_bytes, 50).unwrap();
        assert!(data_uri.starts_with("data:image/jpeg;base64,"));

        // Verify decoded JPEG dimensions are bounded to 50
        let b64 = &data_uri["data:image/jpeg;base64,".len()..];
        let decoded = base64::engine::general_purpose::STANDARD.decode(b64).unwrap();
        let loaded = image::load_from_memory(&decoded).unwrap();
        assert!(loaded.width() <= 50);
        assert!(loaded.height() <= 50);
    }

    #[tokio::test]
    async fn test_prepare_openai_messages_with_images() {
        let mut img = RgbImage::new(10, 10);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([0, 255, 0]);
        }
        let mut png_bytes = Vec::new();
        img.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
            .unwrap();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
        let raw_data_uri = format!("data:image/png;base64,{}", b64);

        let msg = ChatMessageParam::user_with_images(
            "What color is this?",
            vec![raw_data_uri],
        );

        let prepared = prepare_openai_messages(&[msg]).await;
        assert_eq!(prepared.len(), 1);
        let first = &prepared[0];
        assert_eq!(first["role"], "user");

        let content_parts = first["content"].as_array().unwrap();
        assert_eq!(content_parts.len(), 2);
        assert_eq!(content_parts[0]["type"], "text");
        assert_eq!(content_parts[0]["text"], "What color is this?");
        assert_eq!(content_parts[1]["type"], "image_url");
        let url_str = content_parts[1]["image_url"]["url"].as_str().unwrap();
        assert!(url_str.starts_with("data:image/jpeg;base64,"));
    }
}

