# lanshare-rs

📡 A local-network file sharing tool written in Rust.  
Goal: learn Rust + backend systems by building a LAN-only file transfer app.

## Features (MVP)
- Peer discovery via mDNS
- Ask-to-accept transfers
- Large file support with resume & integrity check
- Cross-platform (Linux, macOS, Windows)

## Project Structure
- `lanshare-core/` → networking, transfer logic
- `lanshare-proto/` → protocol definitions
- `lanshare-cli/` → command line interface
- `lanshare-tests/` → integration tests

