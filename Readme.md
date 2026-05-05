# Krypt

A distributed CLI keyring for securely storing credentials and accessing them across devices.

Krypt is built to explore **server-side WebAssembly** using [Spin](https://developer.fermyon.com/spin/overview), a framework for building and deploying microservices as WebAssembly components.

Instead of running a traditional long-lived backend, Spin executes server logic in **isolated, short-lived WASM instances per request**, combining a serverless model with strong isolation.

---

## Quick Start

Get up and running in 2 minutes:

```bash
# Install the CLI
git clone https://github.com/Parth-bathe0142/krypt.git
cd krypt/client
cargo install --path .

# Sign up and start storing credentials
krypt signup
```

---

## Overview

Contains:

* **CLI client** (Rust)
* **Server** (Spin + WebAssembly)
* **Shared crate** (common logic)

Spin is an open-source framework for building and running event-driven microservice applications with WebAssembly (Wasm) components from a number of languages such as Rust, Go, Typescript, Python. Wasm runs in a sandboxed execution environment that can be instanciated within microseconds, enabling an **Instance Per Request (IPR)** model where each incoming request is handled by a fresh, fully isolated WASM instance with its own linear memory.

This approach is conceptually similar to **CGI (Common Gateway Interface)**, but more modern and efficient:

| Feature     | CGI                     | Spin / WASM                   |
| ----------- | ----------------------- | ----------------------------- |
| Execution   | New process per request | New Wasm instance per request |
| Performance | Heavy (spawns multiple processes)   | Lightweight Wasm runtime    |
| Isolation   | Strong                  | Strong                        |
| Portability | Platform dependent binaries       | Runs anywhere               |

---

## Security Model

* Stored keys are **encrypted on the server**
* Encryption is tied to the user's password and updated when the password is changed
* Every request is authenticated (no sessions or tokens)
* Password is stored locally in the system keyring

---

## Repository Structure

```text
krypt/
├── client/            # CLI binary crate
│   ├── src/...
│   ├── build.rs
│   └── Cargo.toml
│
├── server/            # Spin Server application
│   ├── src/...
│   ├── spin.toml      # Spin manifest file
│   └── Cargo.toml
│
├── shared/...         # Validation logic and DTOs
└── Cargo.toml         # Workspace manifest
```

---

## Installation

### From source (development)

```bash
git clone https://github.com/Parth-bathe0142/krypt.git
cd krypt/client
cargo install --path .
```

**Note:** When compiled in debug mode, the CLI makes requests to `localhost:3000` (local dev server) for testing. Release builds use the production URL defined in `build.rs`.

---

## Local Development

### Requirements

* Rust
* Spin CLI
* WASI target:

```bash
rustup target add wasm32-wasip1
```

Install Spin:
[https://developer.fermyon.com/spin/install](https://developer.fermyon.com/spin/install)

---

### Run server locally

```bash
cd server
spin build
spin up
```

This starts a Spin server at `localhost:3000`. You can verify it's running with:

```bash
curl http://localhost:3000/ping
```

Expected response: `pong`

Once the server is running, you can test the CLI in another terminal (in debug mode):

```bash
cd ../client
cargo run -- signup
```

---

## Platform Support

Krypt uses the `dirs` and `keyring` crates with native backends for credential storage:

* **Linux**: `sync-secret-service` (SecretService protocol)
* **macOS**: `apple-native` (Keychain)
* **Windows**: `windows-native` (Windows Credential Manager)

Tested and working on Linux. macOS and Windows support is built-in but not yet verified.

---

## Usage

### Authenticate

```bash
krypt signup
# or
krypt login
```
It will prompt for username and password.

---

### Store a key

```bash
krypt set github
```
It will prompt for the key's value.

---

### List keys

```bash
krypt list
```
Lists all keys stored on the server under the logged in account.

---

### Retrieve a key

```bash
krypt get github
```
Retrieves and displays the value of that key if present.

---

### Update a key

```bash
krypt change github
```
Changes the stored value of the key.

---

### Delete a key

```bash
krypt delete github
```
Removes the key from the server after confirmation.

---

### Account management

```bash
krypt chpassword
krypt logout
krypt delete-account
```

---

## Troubleshooting

### Credentials not stored / prompts for username/password on every use

On each command (except `signup` and `login`), Krypt attempts to:

1. **Fetch username** from the local config file (created during first auth)
2. **Fetch password** from the system keyring using the username

If either fails, an error is displayed and you're prompted to enter the values manually.

**Why this happens:**
- Config file doesn't exist or is unreadable
- System keyring is inaccessible or misconfigured
- Credentials were never stored on first login

**To fix:**
- Ensure the keyring service is running (e.g., `gnome-keyring` on Linux, Keychain on macOS)
- Check file permissions on `~/.config/krypt/config.toml` or equivalent
- Run `krypt login` again to re-store credentials

---

## Limitations & Future Scope

* No account recovery mechanism yet, so losing your password means losing access to stored data.
* Authentication is required on every request. If credentials fail to store, you'll be prompted each time (see Troubleshooting).
* No way to change the server URL to point to a different deployment. This can be moved from `build.rs` to the config file.
* Uses an older version of the `spin_sdk` to maintain compatibility with Fermyon Cloud, which still only supports WASIp1.

---

## Deployment

The server is designed for **Fermyon Cloud**, but can be self-hosted:

* Build with Spin
* Deploy to any supported environment

Self-hosting is recommended if you want full control over your data.

---
