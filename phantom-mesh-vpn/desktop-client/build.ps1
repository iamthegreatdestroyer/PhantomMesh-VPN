# PhantomMesh VPN Desktop Client - Build Script
# This script builds the desktop client for production

param(
    [ValidateSet("windows", "macos", "linux", "all")]
    [string]$Platform = "windows",
    [switch]$Debug,
    [switch]$SkipFrontend
)

$ErrorActionPreference = "Stop"
$DesktopClientDir = $PSScriptRoot

Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  PhantomMesh VPN Desktop Client Builder" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

# Check prerequisites
Write-Host "[1/5] Checking prerequisites..." -ForegroundColor Yellow

# Check Rust
if (!(Get-Command "rustc" -ErrorAction SilentlyContinue)) {
    Write-Error "Rust is not installed. Please install from https://rustup.rs"
    exit 1
}
$rustVersion = rustc --version
Write-Host "  ✓ Rust: $rustVersion" -ForegroundColor Green

# Check Node.js
if (!(Get-Command "node" -ErrorAction SilentlyContinue)) {
    Write-Error "Node.js is not installed. Please install from https://nodejs.org"
    exit 1
}
$nodeVersion = node --version
Write-Host "  ✓ Node.js: $nodeVersion" -ForegroundColor Green

# Check Tauri CLI
if (!(Get-Command "cargo-tauri" -ErrorAction SilentlyContinue)) {
    Write-Host "  Installing Tauri CLI..." -ForegroundColor Yellow
    cargo install tauri-cli
}
Write-Host "  ✓ Tauri CLI: installed" -ForegroundColor Green

# Install frontend dependencies
Write-Host ""
Write-Host "[2/5] Installing frontend dependencies..." -ForegroundColor Yellow

if (!$SkipFrontend) {
    Push-Location "$DesktopClientDir\ui"
    try {
        npm install
        Write-Host "  ✓ NPM packages installed" -ForegroundColor Green
    }
    finally {
        Pop-Location
    }
}
else {
    Write-Host "  ⊘ Skipped (--SkipFrontend)" -ForegroundColor DarkGray
}

# Build frontend
Write-Host ""
Write-Host "[3/5] Building frontend..." -ForegroundColor Yellow

if (!$SkipFrontend) {
    Push-Location "$DesktopClientDir\ui"
    try {
        npm run build
        Write-Host "  ✓ Frontend built successfully" -ForegroundColor Green
    }
    finally {
        Pop-Location
    }
}
else {
    Write-Host "  ⊘ Skipped (--SkipFrontend)" -ForegroundColor DarkGray
}

# Build Tauri app
Write-Host ""
Write-Host "[4/5] Building Tauri application..." -ForegroundColor Yellow

Push-Location $DesktopClientDir
try {
    if ($Debug) {
        cargo tauri build --debug
    }
    else {
        cargo tauri build
    }
    Write-Host "  ✓ Tauri build completed" -ForegroundColor Green
}
finally {
    Pop-Location
}

# Report build outputs
Write-Host ""
Write-Host "[5/5] Build complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Build outputs:" -ForegroundColor Cyan

$bundleDir = "$DesktopClientDir\target\release\bundle"
if ($Debug) {
    $bundleDir = "$DesktopClientDir\target\debug\bundle"
}

# Find installers
$installers = Get-ChildItem -Path $bundleDir -Recurse -Include "*.exe", "*.msi", "*.dmg", "*.AppImage", "*.deb" -ErrorAction SilentlyContinue
foreach ($installer in $installers) {
    $size = [math]::Round($installer.Length / 1MB, 2)
    Write-Host "  → $($installer.FullName) ($size MB)" -ForegroundColor White
}

Write-Host ""
Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  Build successful! Ready for distribution." -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Cyan
