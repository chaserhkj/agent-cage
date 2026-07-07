# AGENTS.md - Agent Cage Development Guide

## Build, Lint, Test

```bash
cargo build --release         # release build (Nix builds via `nix build`)
cargo clippy -- -D warnings   # lint with warnings as errors
cargo fmt -- --check          # check formatting
cargo test                    # no tests exist yet — add #[cfg(test)] in src/
```

Do **not** run the application autonomously (requires Podman/krun sandboxing).

## Config Layering (priority ascending)

1. Embedded `src/defaults.yaml`
2. Contextual: `agent-cage.yaml` / `.agent-cage.yaml` in current or parent dirs (toggled by `--no-contextual-config`)
3. Global: `~/.config/agent-cage.yaml`
4. User-provided: `--config <path>` (toggled by `--no-default-config`)

`--config`, `--no-contextual-config`, `--no-default-config`, and `--dry-run` are global flags (usable before subcommand).

## CLI Structure

```
agent-cage [global flags] run [<profile>] [<instance_name>] [run flags]
agent-cage [global flags] cleanup
```

Default profile: `default`. Instance name is appended last (no flag). See `src/args.rs:40`.

## Recent Features

| Flag | Short | Description |
|------|-------|-------------|
| `--mode` | `-m` | OpMode enum: `disable`, `read-write`, `read-only`, `tmp-overlay`, `tmp-overlay-git` (default), `isolated-git-repo` |
| `--runtime` | `-r` | Default: `krun` |
| `--terminal-connection-type` | `-t` | `direct` PTY (unstable across krun) or `telnet` (default, requires busybox) |
| `--telnet-bind` | `-T` | Default: `127.0.0.1:2323` |
| `--command` | `-C` | Override command (shell-line parsed) |
| `--wrapper-command` | `-W` | Prepend wrappers before command execution |
| `--entrypoint` | `-N` | Override container entrypoint |
| `--volumes` | `-v` | Extra volumes (env var substitution via `subst`) |
| `--envs` | `-e` | Extra env vars |
| `--env-file` | `-E` | Read env file |
| `--hide-fs` | `-H` | Paths (relative to CWD) concealed via anonymous volumes, overlaid last |

## Relative Paths in YAML

Use `!REL!` in config values — it's replaced with the directory of the YAML file. Implemented in `src/rel_provider.rs` via `YamlWithRel`.

## Architecture Notes

- **Entrypoint**: `src/main.rs:11` — parses `Args`, calls `args.exec()`.
- **Profile resolution**: `config.rs:25-41` — merges profile with `global:` block from config hierarchy.
- **Engine config resolution**: `engine.rs:237-266` — layered: base() ← CmdLineEngineConfig defaults (from profile) ← CLI overrides.
- **OpMode volume/working-dir logic**: `args.rs:168-196` — each variant maps to specific volume mounts and workdir.
- **Shell scripts**: embedded via `include_str!()` (`prepare-isolated-git-repo.sh`, `cleanup-isolated-git-repo.sh`).
- **Nix**: `flake.nix` builds via `package.nix`; nixpkgs points to `nixpkgs-unstable`.

## Config YAML Conventions

```yaml
profiles:
  my-agent:
    image: docker.io/author/my-agent
    volumes:
      - ~/.config/my-agent:/root/.config/my-agent:ro
  # global: block under profiles merges into every profile's CmdLineEngineConfig
```

The `Config` struct (`config.rs:19-23`) has `profiles: HashMap<String, Profile>` and an optional `global: CmdLineEngineConfig` — each `Profile` carries its own `cmd_line_config_defaults: CmdLineEngineConfig`.