# AuroraTorrent

AuroraTorrent is a cross-platform BitTorrent client with a Spotify-like UI, built with Rust, Tauri, React, and TypeScript.

## Architecture

```mermaid
graph TD
    UI[Apps/UI (Tauri + React)] <--> Bridge[Crates/Bridge (JSON-RPC)]
    Bridge <--> Engine[Crates/Engine (Core Logic)]
    Engine <--> Network[Internet / P2P Network]
    Engine <--> FS[File System]
```

- **crates/engine**: Core BitTorrent engine (async Rust, Tokio). Handles DHT, peer connections, piece verification, and storage.
- **crates/bridge**: JSON-RPC layer for communication between the UI and the Engine.
- **apps/ui**: The frontend application.
- **examples/seeder**: A standalone seeder for testing.

## Features

- **Spotify-like UI**: Sidebar, Library Grid, Now Playing Footer.
- **Streaming**: Play video/audio as it downloads (HTTP range requests).
- **DHT**: Integrated `mainline` DHT for peer discovery.
- **Cross-Platform**: Windows, macOS, Linux.

## Development

### Prerequisites

- Rust (stable)
- Node.js & npm/pnpm
- System dependencies for Tauri (libwebkit2gtk-4.0-dev, etc. on Linux)

### Running Dev Mode

1. **Start the Engine & UI**:
   ```bash
   # In one terminal
   cd apps/ui
   npm install
   npm run tauri dev
   ```
   *Note: The current scaffold runs the engine embedded or via the bridge. The `tauri dev` command will launch the app.*

2. **Run the Seeder (for testing)**:
   ```bash
   cargo run -p seeder
   ```

## Packaging

### Linux (.deb, AppImage)
```bash
cd apps/ui
npm run tauri build
```

### Windows (.msi)
Run `npm run tauri build` on a Windows machine.

### macOS (.dmg)
Run `npm run tauri build` on a macOS machine.

## License

Dual MIT / Apache-2.0.
