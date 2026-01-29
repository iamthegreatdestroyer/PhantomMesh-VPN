# PhantomMesh VPN Desktop Client

A modern, cross-platform VPN client built with Tauri 2.0, Rust, and React.

## 🚀 Features

- **Cross-Platform**: Windows, macOS, and Linux support
- **Modern UI**: Sleek, glass-morphism design with dark theme
- **System Tray**: Minimize to tray with quick connect/disconnect
- **Kill Switch**: Block all traffic if VPN disconnects unexpectedly
- **Auto-Connect**: Connect automatically on app start
- **Multiple Protocols**: WireGuard (recommended), OpenVPN, Stealth
- **Server Selection**: 50+ locations worldwide
- **Favorites**: Save your preferred servers
- **Split Tunneling**: Exclude specific apps from VPN
- **Connection Stats**: Real-time bandwidth and duration

## 📋 Prerequisites

1. **Rust** (1.70 or later)

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js** (18 or later)

   ```bash
   # Windows (using winget)
   winget install OpenJS.NodeJS

   # macOS (using brew)
   brew install node

   # Linux
   curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
   sudo apt-get install -y nodejs
   ```

3. **Tauri CLI**

   ```bash
   cargo install tauri-cli
   ```

4. **Platform-specific dependencies**:

   **Windows**: WebView2 (usually pre-installed on Windows 10/11)

   **macOS**: Xcode Command Line Tools

   ```bash
   xcode-select --install
   ```

   **Linux (Debian/Ubuntu)**:

   ```bash
   sudo apt update
   sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget \
     file libssl-dev libayatana-appindicator3-dev librsvg2-dev
   ```

## 🛠️ Development Setup

1. **Install frontend dependencies**:

   ```bash
   cd ui
   npm install
   ```

2. **Start development server**:
   ```bash
   # From the desktop-client root directory
   cargo tauri dev
   ```
   This will:
   - Start the Vite dev server on port 1420
   - Build the Rust backend
   - Open the desktop app with hot-reload

## 📦 Building for Production

### Build for current platform

```bash
cargo tauri build
```

### Build outputs:

- **Windows**: `target/release/bundle/nsis/PhantomMesh VPN_x.x.x_x64-setup.exe`
- **macOS**: `target/release/bundle/dmg/PhantomMesh VPN_x.x.x_x64.dmg`
- **Linux**: `target/release/bundle/appimage/PhantomMesh VPN_x.x.x_amd64.AppImage`

## 🏗️ Project Structure

```
desktop-client/
├── Cargo.toml           # Rust dependencies
├── tauri.conf.json      # Tauri configuration
├── build.rs             # Build script
├── src/                 # Rust source code
│   ├── main.rs          # App entry point
│   ├── commands.rs      # Tauri commands (IPC)
│   ├── state.rs         # Application state types
│   ├── error.rs         # Error handling
│   ├── vpn.rs           # VPN client implementation
│   └── tray.rs          # System tray management
└── ui/                  # React frontend
    ├── package.json     # NPM dependencies
    ├── vite.config.ts   # Vite configuration
    ├── tailwind.config.js
    └── src/
        ├── main.tsx     # React entry point
        ├── App.tsx      # Main app component
        ├── index.css    # Global styles
        ├── components/  # React components
        │   └── Layout.tsx
        ├── pages/       # Page components
        │   ├── Dashboard.tsx
        │   ├── Servers.tsx
        │   ├── Settings.tsx
        │   ├── Account.tsx
        │   └── About.tsx
        ├── stores/      # Zustand state
        │   └── vpnStore.ts
        ├── hooks/       # Custom React hooks
        └── utils/       # Utility functions
```

## 🔧 Configuration

### Tauri Configuration (`tauri.conf.json`)

- Window size: 400x700 (compact, mobile-like)
- System tray enabled
- Auto-updater configured
- Platform-specific bundling (NSIS, DMG, AppImage)

### VPN Settings

- **Kill Switch**: Blocks all network traffic if VPN disconnects
- **Auto-Connect**: Connects on startup using Quick Connect logic
- **Protocol**: WireGuard (fastest), OpenVPN (most compatible), Stealth (bypasses firewalls)
- **Split Tunneling**: Configure which apps bypass the VPN

## 🔒 Security Features

1. **Kill Switch Implementation**:
   - Windows: Windows Filtering Platform (WFP)
   - macOS: pf firewall rules
   - Linux: iptables rules

2. **Secure Storage**: Credentials stored using Tauri's secure store plugin

3. **Certificate Pinning**: API communications use pinned certificates

4. **No Logs Policy**: No connection logs stored on client

## 🎨 UI/UX Design

- **Theme**: Dark mode with purple/blue phantom color palette
- **Glass Morphism**: Translucent cards with backdrop blur
- **Animations**: Smooth transitions using Framer Motion
- **Responsive**: Compact layout optimized for desktop
- **Accessibility**: WCAG-compliant contrast ratios

## 📱 Screens

| Screen    | Description                                 |
| --------- | ------------------------------------------- |
| Dashboard | Main view with connect button, stats        |
| Servers   | Server list with search, filters, favorites |
| Settings  | Kill switch, protocol, auto-connect toggles |
| Account   | User info, subscription, logout             |
| About     | App info, version, links                    |

## 🔌 IPC Commands

| Command                | Description                |
| ---------------------- | -------------------------- |
| `connect`              | Connect to specific server |
| `disconnect`           | Disconnect from VPN        |
| `quick_connect`        | Auto-select best server    |
| `get_servers`          | Fetch server list          |
| `get_settings`         | Get current settings       |
| `update_settings`      | Update settings            |
| `get_connection_stats` | Get bandwidth/duration     |
| `toggle_kill_switch`   | Enable/disable kill switch |

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## 📄 License

Copyright © 2025 PhantomMesh. All rights reserved.

See [LICENSE](../LICENSE) for details.
