# code-runner 

A simple Rust-based HTTP service that accepts POST requests at the `/run` endpoint, executes a provided script with a specified language interpreter (e.g. Python or Node.js), and returns the `stdout` and `stderr` as JSON.

## Prerequisites

### Rust and Cargo:
- Install Rust (which includes the Cargo build tool) by following the instructions at [rustup.rs](https://rustup.rs).
- After installation, verify:

```sh
rustc --version
cargo --version
```

### Interpreters (Node.js and Python):
- Ensure you have Node.js and Python installed and accessible on your `PATH`.
- For example:

```sh
node --version
python3 --version
```

- If `python` is not available, but `python3` is, you’ll need to adjust the source code to use `python3` as the interpreter.

### Dependencies:

This project uses:

- Axum for the HTTP server.
- Tokio for asynchronous runtime and process management.
- Serde and serde_json for JSON parsing.
- Tempfile for creating temporary directories.

These are handled by Cargo, so no manual installation is needed beyond having Rust and Cargo.

## Building

### Development Build

Use the default `cargo build` for faster compilation (unoptimized):

```sh
cargo build
```

The binary will be located at `target/debug/code-runner`.

### Production Build

Use the `--release` flag for an optimized production build:

```sh
cargo build --release
```

The binary will be located at `target/release/code-runner`.

## Running

Once built, you can run the server directly from Cargo or by calling the binary.

- From Cargo (dev mode):

  ```sh
  cargo run
  ```

- From Binary (release mode):

  ```sh
  ./target/release/code-runner
  ```

The server will start listening on `0.0.0.0:4000`.

You should see something like:

```csharp
Listening on 0.0.0.0:4000
```

## Testing the Service

### Sample Input

Below is a sample `curl` command to test the `/run` endpoint with a Python script:

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

If Python is accessible as `python` on your system, this should return JSON output similar to:

```json
{
  "stdout": "Hello World from Python\n",
  "stderr": "",
  "exit_code": 0
}
```

If you need to use `python3`, adjust the source code in `src/main.rs`:

```rust
let command = match language {
    "node" => "node",
    "python" => "python3",
    _ => { /* handle error */ }
};
```

Then rebuild and rerun the server.

### Example with Node.js

If testing with Node.js:

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

You should see:

```json
{
  "stdout": "Hello World from Node\n",
  "stderr": "",
  "exit_code": 0
}
```

## Troubleshooting

### Command Execution Error (No such file or directory):

This likely means the specified interpreter (e.g. `python` or `node`) is not found. Make sure you have the interpreter installed and accessible on your `PATH`. You can also specify the full path to the interpreter in the code if needed.

### Permission Issues:

Ensure you have permissions to run executables and write to temporary directories.
