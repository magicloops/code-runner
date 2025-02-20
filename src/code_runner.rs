use axum::{
    extract::Json,
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use tokio::process::Command;
use tempfile::tempdir;
use reqwest;
use std::io;
use chrono::DateTime;

use crate::entropy_reset::reset_entropy_with_bytes;

#[derive(Deserialize)]
pub struct FileSpec {
    name: String,
    content: String,
}

#[derive(Deserialize)]
pub struct Payload {
    language: String,
    files: Vec<FileSpec>,
}

#[derive(Deserialize)]
pub struct RunRequest {
    payload: Payload,
    entropy: Option<String>,
    datetime: Option<String>, 
}

fn update_clock(datetime: &str) -> io::Result<()> {
    let output = std::process::Command::new("date")
        .arg("-s")
        .arg(datetime)
        .output()?;

    if output.status.success() {
        println!("Date set successfully");
    } else {
        eprintln!("Failed to set date: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

pub async fn run_handler(Json(body): Json<RunRequest>) -> (StatusCode, axum::Json<serde_json::Value>) {
    // If entropy is provided, reset it
    if let Some(entropy) = &body.entropy {
        if let Err(e) = reset_entropy_with_bytes(entropy) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({ "error": format!("Failed to reset entropy: {}", e) })),
            );
        }
    }

    // If datetime is provided, update the clock
    if let Some(datetime) = &body.datetime {
        match DateTime::parse_from_rfc3339(datetime) {
            Ok(parsed_time) => {
                let formatted_time = parsed_time.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
                if let Err(e) = update_clock(&formatted_time) {
                    eprintln!("Time sync failed: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(json!({ "error": format!("Failed to update clock: {}", e) })),
                    );
                }
            },
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(json!({ "error": format!("Invalid datetime format: {}", e) })),
                );
            }
        }
    }

    // Identify the language 
    let language = body.payload.language.as_str();

    // Create a temporary directory to store the files
    let dir = match tempdir() {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({ "error": format!("Temp dir error: {}", e) })),
            );
        }
    };

    // Write the files provided to the temp directory
    let mut main_file_path = None;
    for f in &body.payload.files {
        let file_path = dir.path().join(&f.name);
        match std::fs::write(&file_path, f.content.as_bytes()) {
            Ok(_) => {
                if main_file_path.is_none() {
                    main_file_path = Some(file_path);
                }
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(json!({ "error": format!("File write error: {}", e) })),
                );
            }
        }
    }

    let main_file_path = match main_file_path {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({ "error": "No files provided" })),
            );
        }
    };

    match language {
        "python" => {
            // Keep your existing spawn logic for Python
            let mut cmd = Command::new("python3");
            cmd.arg(&main_file_path);

            let output = match cmd.output().await {
                Ok(o) => o,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(json!({ "error": format!("Command execution error: {}", e) })),
                    );
                }
            };

            let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

            let response_json = json!({
                "stdout": stdout_str,
                "stderr": stderr_str,
                "exit_code": output.status.code(),
            });

            (StatusCode::OK, axum::Json(response_json))
        }
        "node" => {
            let code = match std::fs::read_to_string(&main_file_path) {
                Ok(c) => c,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(json!({ "error": format!("Read file error: {}", e) })),
                    );
                }
            };

            let mut body_json = json!({ "code": code });
            
            // Add entropy to the request if it's provided
            if let Some(entropy) = body.entropy {
                body_json["entropy"] = json!(entropy);
            }

            let client = reqwest::Client::new();
            let node_runner_url = "http://127.0.0.1:5000/run";

            let response = match client.post(node_runner_url)
                .json(&body_json)
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(json!({ "error": format!("Failed to reach node-runner: {}", e) })),
                    );
                }
            };

            let response_value: serde_json::Value = match response.json().await {
                Ok(val) => val,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(json!({ "error": format!("Invalid JSON from node-runner: {}", e) })),
                    );
                }
            };

            (StatusCode::OK, axum::Json(response_value))
        }
        _ => {
            (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({ "error": "Unsupported language" })),
            )
        }
    }
}        
