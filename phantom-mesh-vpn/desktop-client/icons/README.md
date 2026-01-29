# PhantomMesh VPN Icons

This directory contains the application icons required for building the desktop client.

## Required Icons

Before building for production, generate the following icons from your source image:

| File | Size | Platform |
|------|------|----------|
| `32x32.png` | 32×32 | Windows, Linux |
| `128x128.png` | 128×128 | Linux |
| `128x128@2x.png` | 256×256 | macOS Retina |
| `icon.ico` | Multi-size | Windows |
| `icon.icns` | Multi-size | macOS |
| `tray-icon.png` | 32×32 or 64×64 | System tray (all platforms) |

## Optional (Windows NSIS Installer)

| File | Size | Description |
|------|------|-------------|
| `header.bmp` | 150×57 | NSIS header image |
| `sidebar.bmp` | 164×314 | NSIS sidebar image |

## Generation

Use the Tauri CLI to generate icons from a source image (minimum 1024×1024 PNG):

```bash
npm run tauri icon path/to/source-icon.png
```

Or use a tool like [RealFaviconGenerator](https://realfavicongenerator.net/) for web-compatible formats.
