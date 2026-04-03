# bugsink-cli

A command-line interface for [Bugsink](https://www.bugsink.com/) error tracking. Designed for both humans and AI agents.

## Installation

### Install script (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/Alfredvc/bugsink-cli/main/install.sh | sh
```

### From source

```bash
cargo install --git https://github.com/Alfredvc/bugsink-cli
```

### From releases

Download the latest binary for your platform from [GitHub Releases](https://github.com/Alfredvc/bugsink-cli/releases).

## Quick Start

```bash
# Interactive login (opens browser for token creation)
bugsink auth login

# Or set environment variables
export BUGSINK_URL="https://your-bugsink-instance.com"
export BUGSINK_TOKEN="your-api-token"

# List projects
bugsink projects list

# List recent issues
bugsink issues list --project 1 --sort last_seen --order desc

# Get a stacktrace
bugsink events stacktrace <event-id>
```

## Authentication

Three ways to authenticate, in priority order:

1. **CLI flags** (per-command): `bugsink --url URL --token TOKEN <command>`
2. **Environment variables**: `BUGSINK_URL` and `BUGSINK_TOKEN`
3. **Config file** (via `bugsink auth login`): stored at `~/.config/bugsink/config.json`

Token creation requires admin access to your Bugsink instance. Non-admin users should ask their administrator for a token.

## Output

All commands output JSON. Pretty-printed in a terminal, compact when piped.

- `--json` — force compact JSON output
- `--fields f1,f2` — filter output to specific fields
- `--all` — fetch all pages (default: first page only)

List commands return a paginated envelope by default:
```json
{"next": "...", "previous": null, "results": [...]}
```

With `--all`, they return a flat array:
```json
[{"id": 1, ...}, {"id": 2, ...}]
```

Errors go to stderr as JSON with exit code 1:
```json
{"error": "message"}
```

## Commands

### Auth
| Command | Description |
|---------|-------------|
| `bugsink auth login` | Interactive authentication |
| `bugsink auth status` | Check if credentials are configured |
| `bugsink auth status --verify` | Verify credentials against the API |
| `bugsink auth logout` | Remove stored credentials |

### Teams
| Command | Description |
|---------|-------------|
| `bugsink teams list` | List all teams |
| `bugsink teams get <id>` | Get team by ID |

### Projects
| Command | Description |
|---------|-------------|
| `bugsink projects list [--team <id>]` | List projects, optionally by team |
| `bugsink projects get <id>` | Get project by ID |
| `bugsink projects create --team <id> --name <name>` | Create a project |

### Issues
| Command | Description |
|---------|-------------|
| `bugsink issues list --project <id> [--sort digest_order\|last_seen] [--order asc\|desc]` | List issues |
| `bugsink issues get <id>` | Get issue by ID |

### Events
| Command | Description |
|---------|-------------|
| `bugsink events list --issue <id> [--order asc\|desc]` | List events for an issue |
| `bugsink events get <id>` | Get event with full data payload |
| `bugsink events stacktrace <id>` | Get stacktrace as markdown |

### Releases
| Command | Description |
|---------|-------------|
| `bugsink releases list --project <id>` | List releases |
| `bugsink releases get <id>` | Get release by ID |
| `bugsink releases create --project <id> --version <version>` | Create a release |

### Schema Discovery
| Command | Description |
|---------|-------------|
| `bugsink describe` | Fetch the full OpenAPI schema |

## Agent Integration

This CLI is designed for AI agent consumption. Key features:

- **JSON-first output** — all commands return structured JSON
- **Predictable error format** — errors on stderr as JSON, exit code 1
- **Schema introspection** — `bugsink describe` returns the full OpenAPI spec at runtime
- **Environment variable auth** — no interactive setup needed in CI/agent contexts
- **Field filtering** — `--fields` reduces token usage by returning only what's needed
- **Pagination control** — `--all` for complete results, default single-page for speed

See [AGENTS.md](AGENTS.md) for the agent-specific reference.

## License

[MIT](LICENSE)
