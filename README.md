# 🌌 AuroraTorrent

<div align="center">

![AuroraTorrent Banner](https://img.shields.io/badge/Aurora-Torrent-00d4ff?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0id2hpdGUiPjxwYXRoIGQ9Ik0xMyAzTDQgMTRoN2wtMSA3IDktMTFoLTdsMS03eiIvPjwvc3ZnPg==)

**A modern, blazing-fast BitTorrent client with streaming support**

Built with Rust, Tauri, React & TypeScript

[![CI](https://github.com/yourusername/AuroraTorrent/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/AuroraTorrent/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/yourusername/AuroraTorrent?include_prereleases)](https://github.com/yourusername/AuroraTorrent/releases)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)

[Download](#-installation) • [Features](#-features) • [Screenshots](#-screenshots) • [Development](#-development) • [Contributing](#-contributing)

</div>

---

## ✨ Features

- **🚀 Real BitTorrent Protocol** - Full implementation of the BitTorrent protocol with DHT, tracker support, and peer wire protocol
- **📺 Stream While Downloading** - Play video and audio files as they download with sequential piece prioritization
- **🎨 Aurora-Themed UI** - Beautiful, modern interface inspired by the northern lights
- **⚡ Lightning Fast** - Built with Rust for maximum performance and minimal resource usage
- **🔒 Private & Secure** - No telemetry, no tracking, your data stays yours
- **🖥️ Cross-Platform** - Native apps for Windows, macOS, and Linux
- **📁 Drag & Drop** - Simply drag .torrent files or magnet links into the app
- **🎯 Smart Piece Selection** - Intelligent piece prioritization for optimal streaming

## 📸 Screenshots

<div align="center">
<table>
<tr>
<td><img src="docs/screenshot-home.png" alt="Home Screen" width="400"/></td>
<td><img src="docs/screenshot-library.png" alt="Library" width="400"/></td>
</tr>
<tr>
<td align="center"><em>Home Dashboard</em></td>
<td align="center"><em>Torrent Library</em></td>
</tr>
</table>
</div>

## 📥 Installation

### Download Pre-built Binaries

Download the latest release for your platform from the [Releases page](https://github.com/yourusername/AuroraTorrent/releases):

| Platform | Download |
|----------|----------|
| **Windows** | `.msi` installer or `.exe` |
| **macOS (Intel)** | `.dmg` disk image |
| **macOS (Apple Silicon)** | `.dmg` disk image (arm64) |
| **Linux** | `.AppImage`, `.deb`, or `.rpm` |

### Build from Source

See the [Development](#-development) section below.

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        AuroraTorrent                            │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │   Frontend   │◄──►│    Bridge    │◄──►│    Engine    │       │
│  │  (React/TS)  │    │  (JSON-RPC)  │    │    (Rust)    │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│         │                   │                   │               │
│         ▼                   ▼                   ▼               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │    Tauri     │    │   Streaming  │    │  BitTorrent  │       │
│  │   WebView    │    │    Server    │    │   Protocol   │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

### Crates

- **`crates/engine`** - Core BitTorrent engine with:
  - Bencode parser
  - Torrent file/magnet link parsing
  - Peer wire protocol
  - HTTP/UDP tracker communication
  - DHT peer discovery (via `mainline`)
  - Piece management with sequential prioritization
  - HTTP streaming server with range request support

- **`crates/bridge`** - JSON-RPC protocol definitions for UI-Engine communication

- **`apps/ui`** - Tauri + React frontend application

## 🛠️ Development

### Prerequisites

- **Rust** (stable, 1.70+)
- **Node.js** (18+) & npm
- **System dependencies** (for Tauri):

  **Ubuntu/Debian:**
  ```bash
  sudo apt install libwebkit2gtk-4.0-dev libwebkit2gtk-4.1-dev \
    libappindicator3-dev librsvg2-dev patchelf libgtk-3-dev
  ```

  **Fedora:**
  ```bash
  sudo dnf install webkit2gtk4.0-devel libappindicator-gtk3-devel librsvg2-devel
  ```

  **macOS:**
  ```bash
  xcode-select --install
  ```

  **Windows:**
  - Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
  - Install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

### Running in Development Mode

```bash
# Clone the repository
git clone https://github.com/yourusername/AuroraTorrent.git
cd AuroraTorrent

# Install frontend dependencies
cd apps/ui
npm install

# Run in development mode (starts both frontend and backend)
npm run tauri dev
```

### Building for Production

```bash
cd apps/ui
npm run tauri build
```

The built application will be in `apps/ui/src-tauri/target/release/bundle/`.

### Running Tests

```bash
# Run all Rust tests
cargo test --all

# Run clippy lints
cargo clippy --all-targets --all-features
```

## 🔧 Configuration

AuroraTorrent stores its configuration and state in:

| Platform | Location |
|----------|----------|
| **Linux** | `~/.config/aurora-torrent/` |
| **macOS** | `~/Library/Application Support/aurora-torrent/` |
| **Windows** | `%APPDATA%\aurora-torrent\` |

### Settings

- **Download Location** - Where downloaded files are saved
- **Max Download Speed** - Bandwidth limit for downloads (0 = unlimited)
- **Max Upload Speed** - Bandwidth limit for uploads (0 = unlimited)

## 📡 Streaming

AuroraTorrent supports streaming media files while they're still downloading:

1. Add a torrent containing video/audio files
2. Click the **Play** button on the torrent card
3. The app will prioritize downloading pieces sequentially for smooth playback
4. The built-in HTTP server (port 3000) serves the content to the video player

### Supported Formats

- **Video:** MP4, MKV, WebM, AVI
- **Audio:** MP3, FLAC, OGG, M4A

*Note: Some formats may require FFmpeg for transcoding (detected automatically)*

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Rust: Follow standard Rust formatting (`cargo fmt`)
- TypeScript: Prettier + ESLint defaults
- Commit messages: Conventional commits preferred

## 📄 License

This project is dual-licensed under either:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

at your option.

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) - For the amazing cross-platform framework
- [mainline](https://github.com/pubky/mainline) - For DHT implementation
- The Rust and React communities

---

<div align="center">
Made with 💜 and lots of ☕

⭐ Star us on GitHub if you find this useful!
</div>
