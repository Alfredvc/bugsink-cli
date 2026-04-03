# Self-Update Command Design

## Overview

Add a `bugsink update` command that checks GitHub Releases for a newer version and replaces the running binary in-place.

## CLI Interface

```
bugsink update
```

Top-level command, no subcommands or flags.

### Output (JSON, consistent with all other commands)

**Already up to date (exit 0):**
```json
{"status": "up_to_date", "version": "0.1.0"}
```

**Updated successfully (exit 0):**
```json
{"status": "updated", "previous_version": "0.1.0", "new_version": "0.2.0"}
```

**Error (exit 1, written to stderr):**
```json
{"error": "message describing what went wrong"}
```

## Mechanism

### Step-by-step flow

1. **Detect current version** — `env!("CARGO_PKG_VERSION")` (compile-time constant).
2. **Fetch latest release** — `GET https://api.github.com/repos/Alfredvc/bugsink-cli/releases/latest` with a `User-Agent: bugsink-cli` header (GitHub API requires it). Parse the `tag_name` field (e.g. `"v0.2.0"`) and strip the `v` prefix.
3. **Compare versions** — Parse both strings as `semver::Version`. If latest <= current, print up-to-date JSON and return.
4. **Detect platform** — Use `std::env::consts::OS` (`linux` or `macos`) and `std::env::consts::ARCH` (`x86_64` or `aarch64`) to build the artifact name `bugsink-{os}-{arch}.tar.gz`. Map `macos` to `darwin` to match release naming.
5. **Download tarball** — Stream the asset from `https://github.com/Alfredvc/bugsink-cli/releases/download/v{version}/bugsink-{os}-{arch}.tar.gz` to a temporary file. The temp file is created in the same directory as the current binary to ensure same-filesystem rename.
6. **Extract binary** — Use `flate2::read::GzDecoder` and `tar::Archive` to extract the `bugsink` binary from the tarball to a temp path in the same directory.
7. **Replace binary** — `std::fs::rename` the extracted binary over `std::env::current_exe().canonicalize()`. This is atomic on the same filesystem. Set executable permissions (0o755) via `std::os::unix::fs::PermissionsExt`.
8. **Clean up** — Remove temp files on both success and failure.

### Platform mapping

| `std::env::consts` | Release artifact |
|---|---|
| OS=`macos`, ARCH=`x86_64` | `bugsink-darwin-x86_64.tar.gz` |
| OS=`macos`, ARCH=`aarch64` | `bugsink-darwin-aarch64.tar.gz` |
| OS=`linux`, ARCH=`x86_64` | `bugsink-linux-x86_64.tar.gz` |
| OS=`linux`, ARCH=`aarch64` | `bugsink-linux-aarch64.tar.gz` |

### No authentication required

GitHub Releases API and asset downloads are public for public repositories. No token needed.

## New Dependencies

| Crate | Purpose |
|---|---|
| `semver` | Parse and compare semantic versions |
| `flate2` | Decompress `.tar.gz` archives |
| `tar` | Extract files from tar archives |

## Error Handling

- **Network failure** (GitHub API or download): report the reqwest error, exit 1.
- **Permission denied** on rename: report the target path and suggest checking permissions, exit 1.
- **No matching asset** for the current platform: report the detected OS/arch and what was expected, exit 1.
- **Corrupt/incomplete download** (tar extraction fails): clean up temp files, report error, exit 1. Original binary is untouched.

No partial-update risk: the old binary remains in place until the atomic rename succeeds.

## Files to Create/Modify

### New files
- `src/commands/update.rs` — Command implementation

### Modified files
- `Cargo.toml` — Add `semver`, `flate2`, `tar` dependencies
- `src/cli.rs` — Add `Update` variant to the `Commands` enum
- `src/main.rs` — Route `Update` command to handler
- `src/commands/mod.rs` — Export `update` module
- `README.md` — Add `bugsink update` to commands table
- `CLAUDE.md` — Add `bugsink update` to commands section

### Test files
- `tests/update.rs` — Integration tests using wiremock

## Testing

- **Unit tests** (in `update.rs`): version comparison logic (current < latest, current == latest, current > latest).
- **Integration tests** (in `tests/update.rs`): mock the GitHub API with wiremock, serve a fake tarball, verify JSON output for both `up_to_date` and `updated` scenarios. Follow existing test patterns in the project.
