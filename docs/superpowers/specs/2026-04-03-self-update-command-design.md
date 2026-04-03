# Self-Update Command Design

## Overview

Add a `bugsink update` command that checks GitHub Releases for a newer version and replaces the running binary in-place.

## CLI Interface

```
bugsink update
```

Top-level command, no subcommands or flags. A check-only mode (`--check`) was considered and deliberately excluded to keep scope minimal.

The `--fields` global flag applies naturally via `output.print()`, consistent with all other commands.

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
2. **Detect installation method** — Resolve `std::env::current_exe().canonicalize()`. If the path contains `.cargo/bin`, exit with an error advising the user to update via `cargo install --git https://github.com/Alfredvc/bugsink-cli` instead. Self-replacing a cargo-managed binary would desync cargo's metadata.
3. **Fetch latest release** — Construct a standalone `reqwest::Client` (not `BugsinkClient`, which is tied to Bugsink API auth). `GET https://api.github.com/repos/Alfredvc/bugsink-cli/releases/latest` with `User-Agent: bugsink-cli` header (GitHub API requires it). If a `GITHUB_TOKEN` env var is set, include it as a `Bearer` token in the `Authorization` header (raises rate limit from 60 to 5000 req/hour). Parse the `tag_name` field (e.g. `"v0.2.0"`) and strip the `v` prefix defensively: `tag_name.strip_prefix('v').unwrap_or(&tag_name)`.
4. **Compare versions** — Parse both strings as `semver::Version`. If latest <= current, print up-to-date JSON and return. Note: the `/releases/latest` endpoint already skips pre-releases and drafts, so pre-release handling is implicit.
5. **Detect platform** — Use `std::env::consts::OS` (`linux` or `macos`) and `std::env::consts::ARCH` (`x86_64` or `aarch64`) to build the artifact name `bugsink-{os}-{arch}.tar.gz`. Map `macos` to `darwin` to match release naming.
6. **Download tarball** — Stream the asset from `https://github.com/Alfredvc/bugsink-cli/releases/download/v{version}/bugsink-{os}-{arch}.tar.gz` to a `tempfile::NamedTempFile` created in the same directory as the current binary (ensures same-filesystem rename). The `tempfile` crate provides automatic cleanup via RAII, including on panic.
7. **Extract binary** — Use `flate2::read::GzDecoder` and `tar::Archive` to extract from the tarball. The release workflow packages via `tar czf ... bugsink`, so the archive contains a single file named `bugsink` at the root (no directory prefix). The extraction must verify that a `bugsink` entry exists and error if the archive structure is unexpected.
8. **Replace binary** — `std::fs::rename` the extracted binary over the resolved exe path. This is atomic on the same filesystem. Set executable permissions (0o755) via `std::os::unix::fs::PermissionsExt`, gated with `#[cfg(unix)]`.
9. **Clean up** — Handled automatically by `tempfile` RAII. Any remaining temp files are cleaned up explicitly on the error path as a safety net.

### Function signature

Unlike other commands, `update` does not use Bugsink API credentials. Its handler signature is:

```rust
pub async fn run(output: &Output) -> Result<()>
```

The routing in `main.rs` calls `update::run(&output).await` without passing `url`, `token`, or `all`.

### Platform support

Only Linux and macOS are supported (matching the release workflow). The `PermissionsExt` usage is gated with `#[cfg(unix)]`. On unsupported platforms, the command returns a clear error: "Self-update is not supported on this platform."

### Platform mapping

| `std::env::consts` | Release artifact |
|---|---|
| OS=`macos`, ARCH=`x86_64` | `bugsink-darwin-x86_64.tar.gz` |
| OS=`macos`, ARCH=`aarch64` | `bugsink-darwin-aarch64.tar.gz` |
| OS=`linux`, ARCH=`x86_64` | `bugsink-linux-x86_64.tar.gz` |
| OS=`linux`, ARCH=`aarch64` | `bugsink-linux-aarch64.tar.gz` |

### GitHub authentication

GitHub Releases API and asset downloads are public for public repositories. No token is required, but the command respects `GITHUB_TOKEN` env var if set (raises the API rate limit from 60 to 5000 requests/hour — relevant for CI/agent environments).

## New Dependencies

| Crate | Purpose |
|---|---|
| `semver` | Parse and compare semantic versions |
| `flate2` | Decompress `.tar.gz` archives |
| `tar` | Extract files from tar archives |
| `tempfile` | RAII temp file management (move from dev-dependency to regular dependency) |

## Error Handling

- **Network failure** (GitHub API or download): report the reqwest error, exit 1.
- **GitHub rate limiting** (403/429 response): detect and report a clear message explaining rate limits, suggest setting `GITHUB_TOKEN`, exit 1.
- **Unexpected API response** (missing `tag_name`, non-semver version): report what was received and what was expected, exit 1.
- **Cargo-installed binary** (path contains `.cargo/bin`): refuse update, advise `cargo install --git ...`, exit 1.
- **Permission denied** on rename: report the target path and suggest checking permissions, exit 1.
- **No matching asset** for the current platform: report the detected OS/arch and what was expected, exit 1.
- **Corrupt/incomplete download** (tar extraction fails or no `bugsink` entry found): clean up temp files, report error, exit 1. Original binary is untouched.
- **Unsupported platform**: report "Self-update is not supported on this platform", exit 1.

No partial-update risk: the old binary remains in place until the atomic rename succeeds.

### Known limitations (v1)

- No checksum or signature verification of downloaded artifacts. Transport security is provided by HTTPS. Application-level integrity verification (e.g., SHA256 checksums alongside releases) is deferred to a future iteration.

## Files to Create/Modify

### New files
- `src/commands/update.rs` — Command implementation

### Modified files
- `Cargo.toml` — Add `semver`, `flate2`, `tar` dependencies; move `tempfile` from dev-dependencies to regular dependencies
- `src/cli.rs` — Add `Update` unit variant to the `Commands` enum (like `Describe`)
- `src/main.rs` — Route `Update` command to `update::run(&output).await` (no url/token/all)
- `src/commands/mod.rs` — Export `update` module
- `README.md` — Add `bugsink update` to commands table
- `CLAUDE.md` — Add `bugsink update` to commands section

### Test files
- `tests/update.rs` — Integration tests using wiremock

## Testing

### Testability

The update command needs injectable parameters for testing:
- **GitHub API URL** — override via `BUGSINK_GITHUB_API_URL` env var (defaults to `https://api.github.com`). Allows wiremock to serve mock responses.
- **Target binary path** — override via `BUGSINK_SELF_PATH` env var (defaults to `std::env::current_exe().canonicalize()`). Allows tests to target a disposable temp file instead of the real test binary.
- **Current version** — override via `BUGSINK_CURRENT_VERSION` env var (defaults to `env!("CARGO_PKG_VERSION")`). Allows tests to simulate version comparisons without rebuilding.

These env vars are test-only affordances. They are not documented in user-facing help.

### Test cases

- **Unit tests** (in `update.rs`): version comparison logic (current < latest, current == latest, current > latest), platform detection mapping, cargo-install detection heuristic.
- **Integration tests** (in `tests/update.rs`):
  - Mock GitHub API with wiremock, return a release with a version equal to the current version, verify `up_to_date` JSON output.
  - Mock GitHub API with a newer version, serve a fake `.tar.gz` containing a dummy `bugsink` binary, set `BUGSINK_SELF_PATH` to a temp file, verify `updated` JSON output and that the temp file was replaced.
  - Mock GitHub API returning 403, verify rate-limit error message.
  - Set `BUGSINK_SELF_PATH` to a path inside `.cargo/bin`, verify cargo-install refusal.
