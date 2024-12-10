use axum::{
    routing::{post},
    extract::Json,
    http::StatusCode,
    Router,
};
use serde::Deserialize;
use serde_json::json;
use std::io::Write;
use tokio::process::Command;
use tempfile::tempdir;
use uuid::Uuid;

#[derive(Deserialize)]
struct FileSpec {
    name: String,
    content: String,
}

#[derive(Deserialize)]
struct Payload {
    language: String,
    files: Vec<FileSpec>,
}

#[derive(Deserialize)]
struct RunRequest {
    image: String,
    payload: Payload,
}

#[tokio::main]
async fn main() {
    // Build our application with a single route
    let app = Router::new().route("/run", post(run_handler));

    // Run it
    let addr = "0.0.0.0:4000".parse().unwrap();
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn run_handler(Json(body): Json<RunRequest>) -> (StatusCode, axum::Json<serde_json::Value>) {
    // Verify the language and determine the command
    let language = body.payload.language.as_str();
    let command = match language {
        "node" => "node",
        "python" => "python3",
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({ "error": "Unsupported language"})),
            );
        }
    };

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

    // Execute the command
    let mut cmd = Command::new(command);
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

    // Convert output to String
    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

    // Return the response as JSON
    let response_json = json!({
        "stdout": stdout_str,
        "stderr": stderr_str,
        "exit_code": output.status.code(),
    });

    (StatusCode::OK, axum::Json(response_json))
}

