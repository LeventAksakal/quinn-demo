# Quinn QUIC Demo

A command-line application that demonstrates QUIC protocol communication using the Quinn library for Rust. This demo implements both a client and server to showcase bidirectional data exchange over QUIC.

## Features

- QUIC server that accepts connections and handles multiple clients
- QUIC client that connects to the server and exchanges messages
- Command-line interface with multiple modes:
  - Server mode: Run only the server component
  - Client mode: Run only the client component
  - Demo mode: Run both server and client for demonstration

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Certificate and key files (included for local testing)

### Installation

Clone the repository and build the project:

```bash
git clone https://github.com/yourusername/quinn-demo.git
cd quinn-demo
cargo build --release
```

### Usage

- Run in demo mode (starts both server and client) with `cargo run`
- Run in server mode with `cargo run -- server`
- Run in client mode with `cargo run -- client`

### Command-line Options

- Run `cargo run -- help`

## Security Note

- The included certificate and key files are for development purposes only. In a production environment use properly signed certificates.

