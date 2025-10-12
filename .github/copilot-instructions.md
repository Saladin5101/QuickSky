# QuickSky AI Coding Agent Instructions

Welcome to the QuickSky codebase! This document provides essential guidance for AI coding agents to be productive in this repository. QuickSky is a standalone, native version control tool designed to simplify workflows for developers. Below are the key aspects of the project structure, workflows, and conventions.

---

## Project Overview

QuickSky is a **native version control tool** built from scratch. It eliminates the need for external tools like Git and focuses on developer-friendly workflows. The project is written in **Rust** with platform-specific C integrations for low-level system operations.

### Key Components
- **`src/`**: Core Rust implementation of QuickSky.
  - `cli/`: Command-line interface logic.
  - `ffi/`: Foreign Function Interface for C integrations.
  - `remote/`: Handles remote repository interactions.
  - `repo/`: Manages local repository operations.
- **`c_src/`**: Platform-specific C code for system-level operations.
  - `macos/`, `linux/`, `windows/`: OS-specific implementations.
- **`examples/`**: Demonstration projects showcasing QuickSky usage.
- **`docs/`**: Documentation for developers and users.
- **`packaging/`**: Scripts and files for creating platform-specific installers.

---

## Developer Workflows

### Building the Project
QuickSky uses **Cargo** for building the Rust codebase. To include platform-specific C code, ensure you have the appropriate build tools installed (e.g., `clang` for macOS).

```bash
cargo build
```

For release builds:
```bash
cargo build --release
```

### Running Tests
Tests are written in Rust and can be executed using Cargo:
```bash
cargo test
```

### Debugging
Use `lldb` (macOS) or `gdb` (Linux) for debugging native code. For Rust code, `rust-lldb` is recommended.

### Platform-Specific Builds
To build QuickSky for a specific platform, navigate to the corresponding directory in `c_src/` and ensure the appropriate compiler is used. For example, on macOS:
```bash
cd c_src/macos
clang -o sys_macos sys_macos.c
```

---

## Project-Specific Conventions

### Code Style
- **Rust**: Follow the `rustfmt` conventions. Run `cargo fmt` before committing.
- **C**: Use K&R style for consistency across platform-specific code.

### Error Handling
- Use `anyhow` for error propagation in Rust.
- Platform-specific C code should return error codes and log errors to `stderr`.

### Logging
- Use the `log` crate in Rust for structured logging.
- Platform-specific C code logs to `syslog` (Linux/macOS) or `Event Viewer` (Windows).

---

## Integration Points

### External Dependencies
- **Rust Crates**: `anyhow`, `clap`, `log`.
- **C Libraries**: Standard system libraries (e.g., `libc`).

### Cross-Component Communication
- Rust and C communicate via FFI. See `src/ffi/` for bindings.
- Remote repository interactions are abstracted in `src/remote/`.

---

## Examples

### Adding a New Command
1. Define the command in `src/cli/`.
2. Implement the logic in `src/`.
3. Update the CLI parser in `src/main.rs`.

### Modifying Platform-Specific Code
1. Edit the relevant file in `c_src/<platform>/`.
2. Ensure the changes are compatible with the FFI bindings in `src/ffi/`.
3. Test on the target platform.

---

## Notes
- QuickSky is licensed under AGPLv3. Ensure all contributions comply with this license.
- For questions or issues, refer to the [README](../README.md) or open an issue on GitHub.

---

This document is a living guide. Update it as the project evolves.