# OpenCode Standalone Desktop

A standalone desktop app for OpenCode that works with a pre-installed `opencode` CLI in your PATH, rather than bundling it into the binary.

## Building

Requires Docker. No local Rust, Bun, or system libraries needed.

```bash
./build.sh
```

For a clean build without Docker cache:

```bash
./build.sh --no-cache
```

Artifacts will be in `build/`:

| File                 | Description           |
| -------------------- | --------------------- |
| `OpenCode`           | Linux binary          |
| `OpenCode Dev_*.deb` | Debian/Ubuntu package |
| `OpenCode Dev-*.rpm` | Fedora/RHEL package   |

## Development

```bash
bun install
bun run --cwd packages/desktop tauri dev
```

## Usage

The built app expects `opencode` to be available in your PATH. Install it separately before launching the desktop app.
