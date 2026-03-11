# AGENTS.md - Agent Cage Development Guide

## Overview

Agent Cage is a Rust CLI tool that executes AI coding agents within an isolated container environment using Podman. It provides configurable profiles, multiple isolation modes, and secure sandboxing for agent operations.

## Project Structure

```
/work
├── Cargo.toml           # Rust package manifest
├── src/
│   ├── main.rs          # Entry point
│   ├── args.rs          # CLI argument parsing (clap)
│   ├── config.rs        # Configuration loading (figment)
│   ├── engine.rs        # Container engine execution
│   ├── rel_provider.rs  # Relative path provider for config
│   ├── utils.rs         # Utility functions
│   └── defaults.yaml    # Default configuration
└── README.md
```

## Build, Lint, and Test Commands

### Building the Project

```bash
# Debug build
cargo build

# Release build (recommended for installation)
cargo build --release

# Build with specific number of parallel jobs
cargo build --release -j 4
```

### Linting and Formatting

```bash
# Run clippy lints
cargo clippy

# Run clippy with warnings as errors
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting without making changes
cargo fmt -- --check
```

### Running Tests

```bash
# Run all tests
cargo test

# Run a single test by name
cargo test test_name

# Run tests with output displayed
cargo test -- --nocapture

# Run doc tests
cargo test --doc
```

Note: This project currently has no test suite. When adding tests, place them in `src/` with `#[cfg(test)]` modules or in a `tests/` directory.

### Running the Application

Agents SHOULD NOT attempt to run this application autonomously, since it requires sandboxing facilities and is most likely to fail when run from agentic contexts.

Focus on running unit tests only. When the user asks you to run the application or do end to end tests, kindly remind them that this application need to be end tested manually.

## Code Style Guidelines

### General Principles

- Use Rust 2024 edition (set in Cargo.toml)
- Prefer explicit error handling with `anyhow::Result`
- Keep functions small and focused
- Add doc comments for public APIs

### Imports

```rust
// Standard library imports first
use std::path::Path;

// Then external crate imports (alphabetical)
use anyhow::{Context, Result};
use clap::Parser;

// Then local imports
use crate::config::Config;
```

### Formatting

- Use 4 spaces for indentation (Rust default)
- Maximum line length: 100 characters (soft guideline)
- Use trailing commas in struct literals and match arms
- Keep related items together

### Types and Naming

```rust
// Types: PascalCase
struct Profile { ... }
enum OpMode { ... }
type Result<T> = anyhow::Result<T>;

// Functions and variables: snake_case
fn get_merged_profile(&self, name: &str) -> Option<Profile> {
    let mut config = Vec::new();
}

// Constants: SCREAMING_SNAKE_CASE
const MAX_RETRY_COUNT: u32 = 3;

// Enums variants: PascalCase
pub enum OpMode {
    Disable,
    ReadWrite,
    TmpOverlay,
}
```

### Error Handling

- Use `anyhow::Result<T>` for application code
- Use `.context()` to add context to errors
- Return early on errors when possible

```rust
fn example() -> Result<()> {
    let config = parse_config()
        .context("Failed to parse configuration")?;
    Ok(())
}
```

### Structs and Enums

- Use `#[derive(Debug, Clone, Serialize, Deserialize)]` for data structures
- Use `#[skip_serializing_none]` for optional fields in serialized structs
- Use `#[serde(flatten)]` for flattening nested configs

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    profiles: HashMap<String, Profile>,
    #[serde(flatten)]
    pub cmd_line_config_defaults: CmdLineEngineConfig,
}
```

### CLI Arguments

- Use `clap` with derive macros
- Group related arguments with `#[command(flatten)]`
- Use `ValueEnum` for enum-like arguments

```rust
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    #[command(subcommand)]
    sub_command: SubCommand,
}
```

### Macros

- Use existing macros (like `define_resolvable_struct!` in args.rs) for repetitive patterns
- Keep macro invocations clean and well-documented

### Configuration

- Use `figment` for configuration with multiple providers
- Support layering: default → contextual → user config
- Use YAML for configuration files

### Container Engine

- Currently only supports Podman
- Runtime defaults to `krun`
- Use builder pattern for `EngineConfig`

```rust
EngineConfig {
    image: self.image.to_owned(),
    name: None,
    cmd_line_config: parsed_config.resolve(&self.cmd_line_config_defaults)?,
    ephemeral: false,
}
```

### Shell Scripts

- Place helper scripts in `src/` and include with `include_str!()`
- Use `utils::run_in_foreground()` to execute external commands

## Dependencies

Key dependencies (check Cargo.toml for versions):
- `anyhow` - Flexible error handling
- `clap` - CLI argument parsing
- `figment` - Configuration management
- `serde` / `serde_with` - Serialization
- `shell-words` - Shell command parsing
- `subst` - Environment variable substitution

## Common Development Tasks

### Adding a New Profile

1. Add profile to `src/defaults.yaml`
2. Define fields in `config.rs` `Profile` struct
3. Add CLI args in `args.rs` if needed

### Adding a New Operation Mode

1. Add variant to `OpMode` enum in `args.rs`
2. Implement `to_volume_mounts()` and `to_work_dir()`
3. Handle mode in `engine.rs`

### Adding a New CLI Command

1. Add variant to `SubCommand` enum in `args.rs`
2. Implement execution logic in `Args::exec()`

## Notes

- This project uses Podman as the container runtime
- Default operation mode is `tmp-overlay-git` for safety
- Terminal connection defaults to telnet for krun compatibility
