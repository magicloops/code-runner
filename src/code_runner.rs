use axum::{
    extract::Json,
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use tokio::process::Command;
use tempfile::tempdir;
use reqwest;

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
    //image: String, //Legacy from past code-runner
    payload: Payload,
}

pub async fn run_handler(Json(body): Json<RunRequest>) -> (StatusCode, axum::Json<serde_json::Value>) {
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
                // If there's only one file or we assume the first file is the main file:
                // For simplicity, let's assume the first provided file is the entrypoint.
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

    // Depending on the language, either call Python or forward to Node-runner
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
            // 1) Read the JS code from main_file_path
            // 2) Send it to Node-runner on localhost:5000
            let code = match std::fs::read_to_string(&main_file_path) {
                Ok(c) => c,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(json!({ "error": format!("Read file error: {}", e) })),
                    );
                }
            };

            // Construct JSON body to send
            let body_json = json!({ "code": code });

            // Make an HTTP POST to node-runner
            let client = reqwest::Client::new();
            let node_runner_url = "http://127.0.0.1:5000/run";

            // Send request
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

            // Parse the JSON response from node-runner
            let response_value: serde_json::Value = match response.json().await {
                Ok(val) => val,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(json!({ "error": format!("Invalid JSON from node-runner: {}", e) })),
                    );
                }
            };

            // Return node-runner's response back to the client
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

