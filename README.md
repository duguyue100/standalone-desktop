# AlfAlfa

**ALFALFA: LatticeFlow's Awesome Little Friendly Agent**

A desktop application for working with LatticeFlow AI GO! evaluations, powered by [OpenCode](https://opencode.ai).

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/duguyue100/standalone-desktop/main/install.sh | bash
```

This downloads the latest release binary to `~/.local/bin/AlfAlfa` and checks that dependencies are available.

### Prerequisites

AlfAlfa requires two CLI tools in your PATH before launching:

| Dependency | Purpose | Install |
|------------|---------|---------|
| `opencode` | AI coding agent (the backend server) | [opencode.ai](https://opencode.ai) |
| `lf` | LatticeFlow AI GO! CLI | Activate the Python venv that provides it |

## Usage

Launch from a terminal with both dependencies available:

```bash
alfalfa
```

On first launch, AlfAlfa creates `~/.alfalfa/` which holds its configuration, auth, and the custom `lf` tool. Session data is isolated from your personal OpenCode installation.

### Features

- **Alfalfa identity** -- custom system prompts for LatticeFlow evaluation workflows (build and plan modes)
- **`lf` tool** -- dedicated tool for LatticeFlow CLI operations (preferred over raw bash)
- **`lf skills`** -- LatticeFlow CLI knowledge base injected into every new session
- **Project journal** -- per-project journal at `.lf_agent/journal/` for tracking evaluation progress
  - `/journal` -- update the journal from the current session
  - `/journal-review` -- review the journal for gaps, stale info, contradictions

### Data directories

| Path | Contents |
|------|----------|
| `~/.alfalfa/` | OpenCode server data, config, auth, custom tools |
| `<project>/.lf_agent/journal/` | Per-project journal files |

## Building from source

Requires Docker. No local Rust, Bun, or system libraries needed.

```bash
./build.sh              # build
./build.sh --install    # build and copy to ~/.local/bin
./build.sh --no-cache   # clean Docker build
```

Artifacts in `build/`:

| File | Description |
|------|-------------|
| `alfalfa` | Linux x86_64 binary |
| `AlfAlfa Dev_*.deb` | Debian/Ubuntu package |
| `AlfAlfa Dev-*.rpm` | Fedora/RHEL package |

## Development

```bash
bun install
bun run dev
```

## Releasing

Tag a version and push to trigger the CI build:

```bash
git tag v0.2.0
git push origin main --tags
```

This builds Linux x86_64 and macOS aarch64 binaries and publishes them as a GitHub release.
