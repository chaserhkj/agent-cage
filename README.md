# agent-cage

Run your coding agent inside an isolated sandbox.

## Description

agent-cage is a Rust CLI tool that executes AI coding agents (such as Aider or Opencode) within an isolated container environment using Podman. It provides configurable profiles, multiple isolation modes, and secure sandboxing for agent operations.

## Features

- **Multiple isolation modes**: Read-write, read-only, ephemeral overlay, and isolated git repo modes
- **Profile-based configuration**: Define different agent setups with custom images, volumes, and environment variables
- **Terminal flexibility**: Support for direct PTY or telnet-based terminal connections
- **Configuration layering**: Merge default, contextual, and user-provided configurations
- **Built-in profiles**: Pre-configured profiles for Aider and Opencode agents

## Installation

```bash
cargo build --release
```

## Usage

```bash
# Run with default profile
agent-cage run

# Run with specific profile
agent-cage run opencode

# Run with custom config file
agent-cage run --config config.yaml

# Run with ephemeral overlay mode
agent-cage run --mode tmp-overlay

# Use telnet terminal connection
agent-cage run --terminal-connection-type telnet --telnet-bind 127.0.0.1:2323

# Cleanup isolated git repo
agent-cage cleanup
```

## Configuration

Configuration is loaded from multiple sources in order of precedence:
1. Default configuration (`defaults.yaml`)
2. Contextual configurations (`agent-cage.yaml` in parent directories)
3. User-provided config file (`--config`)

Example configuration:

```yaml
profiles:
  my-agent:
    image: nixery.dev/shell/nix/busybox/my-agent
    envs:
      - HOME=/root
      - USER=root
    volumes:
      - ~/.config/my-agent:/root/.config/my-agent:ro
```

## Operation Modes

- `disable`: No volume mounts
- `read-write`: Full read-write access to current directory
- `read-only`: Read-only access to current directory
- `tmp-overlay`: Ephemeral overlay, changes discarded on exit
- `tmp-overlay-git`: Ephemeral overlay on `.git` directory only
- `isolated-git-repo`: Creates isolated nested git repository

## License

MIT
