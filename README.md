# code-runner

A simple Rust-based HTTP service that accepts POST requests at the `/run` endpoint, executes a provided script with a specified language interpreter (e.g., Python or Node.js), and returns the `stdout` and `stderr` as JSON.  
Additionally, this repository contains a Node.js script (`node-runner.js`) that can be run alongside the Rust service for handling certain JavaScript execution scenarios more efficiently.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Project Layout](#project-layout)
3. [Building](#building)
   1. [Development Build](#development-build)
   2. [Production Build](#production-build)
4. [Running Locally](#running-locally)
5. [Testing the Service](#testing-the-service)
   1. [Sample Input (Python)](#sample-input-python)
   2. [Sample Input (Node.js)](#sample-input-nodejs)
6. [Running via Docker](#running-via-docker)
7. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Rust and Cargo
- Install Rust (which includes Cargo) from [rustup.rs](https://rustup.rs).
- Verify installation:
  ```sh
  rustc --version
  cargo --version
  ```

### Interpreters (Node.js and Python)
- Ensure you have Node.js and Python installed and accessible on your `PATH`.
  ```sh
  node --version
  python3 --version
  ```
- If `python` is not available but `python3` is, you may need to adjust the source to invoke `python3` directly (see [Testing the Service](#testing-the-service)).

### PDF Parsing Libraries
Both runtimes ship with PDF parsing packages preinstalled:
- **Python**: [`PyPDF2`](https://pypi.org/project/PyPDF2/)
- **Node.js**: [`pdf-parse`](https://www.npmjs.com/package/pdf-parse)

These allow user code to extract text from PDF files without installing additional dependencies.

### Dependencies
This project uses:
- **Axum** for the HTTP server
- **Tokio** for asynchronous runtime
- **Serde** and **serde_json** for JSON parsing
- **Tempfile** for creating temporary directories

Cargo handles all Rust dependencies automatically; no extra manual steps are required once you have Rust and Cargo installed.

---

## Project Layout

```
.
├─ Dockerfile          # Docker build steps
├─ src/
│  └─ main.rs         # Rust-based HTTP server
├─ node-runner/
│  ├─ node-runner.js  # Node.js script for handling JS requests
│  ├─ package.json
│  └─ package-lock.json
├─ Cargo.toml
└─ README.md
```

- **`src/`**: Contains the Rust service source code (`main.rs`).
- **`node-runner/`**: Contains the Node.js runner script (`node-runner.js`) and its `npm` dependencies.

---

## Building

### Development Build

Build in dev mode (faster compilation, unoptimized):

```sh
cargo build
```

The binary is located at `target/debug/code-runner`.

### Production Build

Build in release mode (optimized):

```sh
cargo build --release
```

The binary is located at `target/release/code-runner`.

---

## Running Locally

1. **Rust Service**  
   From the repository root:
   ```sh
   cargo run
   ```
   This starts the Rust service on `0.0.0.0:4000`.

2. **Node Runner (Optional)**  
   If you need to run the separate Node.js script (for debugging or local testing), do the following in a separate terminal:
   ```sh
   cd node-runner
   npm install
   node node-runner.js
   ```
   By default, this listens on a different port (5000) and simply run JavaScript tasks.

---

## Testing the Service

Below are sample `curl` requests to test the `/run` endpoint exposed by the Rust-based HTTP service.

### Sample Input (Python)

```sh
curl -X POST http://localhost:4000/run \
  -H 'Content-Type: application/json' \
  -d '{
        "image": "python:latest",
        "payload": {
          "language": "python",
          "files": [
            {
              "name": "script.py",
              "content": "print(\"Hello World from Python\")"
            }
          ]
        }
      }'
```

If everything is configured correctly (and your system has `python`), you should see JSON similar to:

```json
{
  "stdout": "Hello World from Python\n",
  "stderr": "",
  "exit_code": 0
}
```

> **Note**: If `python` isn’t on your PATH, but `python3` is, you can either:
> - Add an alias for `python`, or  
> - Adjust the interpreter command in `src/main.rs`:
>   ```rust
>   match language {
>       "python" => "python3",
>       _ => { /* handle error */ }
>   };
>   ```
>   Then rebuild and rerun.

### Sample Input (Node.js)

```sh
curl -X POST http://localhost:4000/run \
  -H 'Content-Type: application/json' \
  -d '{
        "image": "node:latest",
        "payload": {
          "language": "node",
          "files": [
            {
              "name": "script.js",
              "content": "console.log(\"Hello World from Node\");"
            }
          ]
        }
      }'
```

Expected output:

```json
{
  "stdout": "Hello World from Node\n",
  "stderr": "",
  "exit_code": 0
}
```

---

## Running via Docker

1. **Build the Docker image**  
   From the repository root (where the `Dockerfile` resides):
   ```sh
   docker build -t code-runner-service:container .
   ```
   This multi-stage build installs Rust, builds the `code-runner` binary, installs any Node.js dependencies in `node-runner/`, and then copies them into a minimal final image.

2. **Run the container**  
   ```sh
   docker run -p 4000:4000 code-runner-service:container
   ```
   This starts the Rust HTTP service on port 4000 and also spawns `node-runner.js` in the background (depending on your Dockerfile setup). If everything is correct, you can send requests to `http://localhost:4000/run`.

---

## Troubleshooting

### Command Execution Error (No such file or directory)
If you see an error like:
```
No such file or directory
```
This usually indicates the specified interpreter (e.g. `python` or `node`) is not on the `PATH`. Make sure you have the interpreter installed and accessible. In a Docker environment, confirm the Dockerfile includes the relevant installations.

### Permission Issues
Ensure you have permissions to run executables and write to temporary directories. On Unix-like systems, ensure the binary (`code-runner`) has the executable bit set, and that Docker has permissions to copy and run it.

---

Happy hacking!
