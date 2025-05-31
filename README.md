# DVD Ripper

A high-performance, cross-platform DVD ripping application written in Rust with both GUI and CLI interfaces.

## 🚀 Features

- **Cross-Platform**: Works on Linux, macOS, and Windows
- **Multi-threaded**: Efficient parallel processing for fast ripping
- **Dual Interface**: Modern GUI with egui or command-line interface
- **HandBrake Integration**: Leverages HandBrake CLI for reliable transcoding
- **Remote Upload**: Built-in support for rsync, scp, and sftp uploads
- **DVD Detection**: Automatic DVD detection and mounting
- **Title Scanning**: Comprehensive DVD structure analysis
- **Production Ready**: Robust error handling and logging

## 📦 Installation

### Pre-built Binaries

Download the latest release for your platform from the [releases page](https://github.com/sleepyyui/copydvd/releases):

- **Linux**: `dvd_ripper-linux-x86_64` or `dvd_ripper-linux-aarch64`
- **macOS**: `dvd_ripper-macos-x86_64` (Intel) or `dvd_ripper-macos-aarch64` (Apple Silicon)
- **Windows**: `dvd_ripper-windows-x86_64.exe`

Make the binary executable on Unix systems:
```bash
chmod +x dvd_ripper-*
```

### Build from Source

#### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- HandBrake CLI (`HandBrakeCLI` in PATH)
- Platform-specific dependencies:
  - **Linux**: `pkg-config`, `libx11-dev`, `libxrandr-dev`, `libxi-dev`, `libgl1-mesa-dev`
  - **macOS**: Xcode command line tools
  - **Windows**: Visual Studio Build Tools

#### Build Steps

```bash
git clone https://github.com/your-repo/copydvd.git
cd copydvd
cargo build --release
```

The binary will be available at `target/release/dvd_ripper` (or `.exe` on Windows).

## 🖥️ Usage

### GUI Mode (Default)

Simply run the application to launch the graphical interface:

```bash
./dvd_ripper
```

The GUI provides:
- DVD detection and scanning
- Title selection and preview
- Progress tracking
- Configuration management
- Upload functionality

### CLI Mode

Use the `--cli` flag for command-line operation:

```bash
# Basic ripping
./dvd_ripper --cli --input /dev/dvd --output ./ripped_movies

# With specific settings
./dvd_ripper --cli \
  --input /dev/dvd \
  --output ./movies \
  --quality 20 \
  --preset "Fast 1080p30" \
  --threads 4

# Upload after ripping
./dvd_ripper --cli \
  --input /dev/dvd \
  --output ./movies \
  --upload \
  --server example.com \
  --username myuser \
  --remote-path /media/movies
```

### Command Line Options

- `--cli`: Enable CLI mode
- `--input <PATH>`: DVD source path (default: auto-detect)
- `--output <PATH>`: Output directory
- `--quality <NUM>`: Video quality (0-51, lower = better)
- `--preset <NAME>`: HandBrake preset name
- `--threads <NUM>`: Number of concurrent ripping threads
- `--upload`: Enable upload after ripping
- `--server <HOST>`: Upload server hostname
- `--username <USER>`: Upload username
- `--remote-path <PATH>`: Remote directory path
- `--method <METHOD>`: Upload method (rsync, scp, sftp)

## 🔧 Configuration

Configuration is stored in platform-specific locations:
- **Linux**: `~/.config/dvd_ripper/config.json`
- **macOS**: `~/Library/Application Support/dvd_ripper/config.json`
- **Windows**: `%APPDATA%/dvd_ripper/config.json`

Example configuration:
```json
{
  "handbrake_preset": "Fast 1080p30",
  "video_quality": 20,
  "max_threads": 4,
  "output_directory": "/home/user/Movies",
  "auto_upload": false,
  "server_config": {
    "hostname": "myserver.com",
    "username": "myuser",
    "remote_path": "/media/movies",
    "method": "rsync"
  }
}
```

## 🚀 Automated Builds & Releases

This project features a comprehensive CI/CD pipeline that automatically builds and releases binaries for all supported platforms.

### Release Process

1. **Automatic Releases**: Pushes to the `v2` branch trigger automated builds and releases
2. **Cross-Platform Builds**: Binaries are built for Linux (x86_64, aarch64), macOS (Intel, Apple Silicon), and Windows (x86_64)
3. **Changelog Generation**: Release notes are automatically generated from commit messages
4. **Asset Upload**: All platform binaries are uploaded as release assets

### Version Management

Use the provided script to bump versions:

```bash
# Bump patch version (0.1.0 -> 0.1.1)
./scripts/bump_version.sh patch

# Bump minor version (0.1.0 -> 0.2.0)
./scripts/bump_version.sh minor

# Bump major version (0.1.0 -> 1.0.0)
./scripts/bump_version.sh major
```

After bumping, commit and push to `v2` branch:
```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to X.Y.Z"
git push origin v2
```

### CI/CD Workflows

- **Release Workflow** (`.github/workflows/release.yml`): Builds and releases on `v2` branch
- **CI Workflow** (`.github/workflows/ci.yml`): Tests and validation on all other branches
- **Dependency Updates** (`.github/workflows/dependencies.yml`): Weekly automated dependency updates

## 🛠️ Development

### Project Structure

```
src/
├── main.rs          # Application entry point
├── app/             # Application state and runner
├── cli.rs           # Command-line interface
├── config.rs        # Configuration management
├── dvd/             # DVD detection, mounting, and ripping
├── error.rs         # Error types and handling
├── gui.rs           # GUI implementation (egui)
├── upload.rs        # Upload functionality
└── utils.rs         # Utility functions
```

### Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Run tests: `cargo test --all-features`
5. Run clippy: `cargo clippy --all-targets --all-features -- -D warnings`
6. Format code: `cargo fmt --all`
7. Commit changes: `git commit -m 'feat: add amazing feature'`
8. Push to branch: `git push origin feature/amazing-feature`
9. Open a Pull Request

### Running Tests

```bash
# Run all tests
cargo test --all-features

# Run tests without GUI
cargo test --no-default-features

# Run with coverage
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

## 📋 Requirements

### System Requirements

- **OS**: Linux, macOS 10.15+, or Windows 10+
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: Sufficient space for ripped content (DVDs are typically 4-8GB)
- **DVD Drive**: Physical DVD drive or mounted ISO files

### Dependencies

- **HandBrake CLI**: Required for transcoding
  - Linux: `apt install handbrake-cli` or similar
  - macOS: `brew install handbrake`
  - Windows: Download from [handbrake.fr](https://handbrake.fr/)

### Optional Dependencies

- **rsync**: For rsync uploads (usually pre-installed on Unix systems)
- **openssh-client**: For scp/sftp uploads
- **sshpass**: For password-based SSH authentication

## 🚨 Troubleshooting

### Common Issues

1. **DVD Not Detected**
   - Ensure DVD is properly inserted and mounted
   - Check permissions for DVD device access
   - Verify DVD is not copy-protected beyond HandBrake's capabilities

2. **HandBrake Not Found**
   - Install HandBrake CLI and ensure it's in PATH
   - Test with: `HandBrakeCLI --version`

3. **Permission Errors**
   - Run with appropriate permissions for DVD access
   - On Linux: Add user to `cdrom` group

4. **Upload Failures**
   - Verify network connectivity and credentials
   - Check SSH key setup for key-based authentication
   - Ensure remote directory exists and is writable

### Logs

Application logs are written to:
- **Linux**: `~/.local/share/dvd_ripper/logs/`
- **macOS**: `~/Library/Logs/dvd_ripper/`
- **Windows**: `%LOCALAPPDATA%/dvd_ripper/logs/`

Enable debug logging with:
```bash
RUST_LOG=debug ./dvd_ripper
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚖️ Legal Notice

**Educational Use Only** - This repository/code is provided strictly for **educational purposes only**. It is designed to help understand concepts and techniques in a learning environment.

### Important Notice

- **Legal Use Only**: Users must comply with all local, state, national, and international laws and regulations
- **No Illegal Activities**: This content may not be used for any illegal, harmful, or malicious purposes
- **No Warranty**: This code is provided "as is" without any warranties of any kind
- **User Responsibility**: The end user assumes full responsibility for how they use this content

By accessing or using this repository/code, you agree to use it solely for legitimate educational purposes and in accordance with all applicable laws.

## 🙏 Acknowledgments

- [HandBrake](https://handbrake.fr/) for the excellent transcoding engine
- [egui](https://github.com/emilk/egui) for the immediate mode GUI framework
- [tokio](https://tokio.rs/) for async runtime
- The Rust community for excellent crates and tools
