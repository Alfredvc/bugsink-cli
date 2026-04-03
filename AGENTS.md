# bugsink-cli

CLI tool for interacting with Bugsink error tracking. Designed for AI agents.

## Authentication

Set environment variables (recommended for agents):
```bash
export BUGSINK_URL="https://your-bugsink-instance.com"
export BUGSINK_TOKEN="your-api-token"
```

Or pass per-command: `bugsink --url URL --token TOKEN <command>`

Or run interactive setup: `bugsink auth login`

## Output Format

All commands output JSON. When stdout is a terminal, JSON is pretty-printed.
Use `--json` to force compact JSON. Use `--fields f1,f2` to filter output fields.
Use `--all` to fetch all pages of paginated results (default: first page only).

List commands return `{"next": ..., "previous": ..., "results": [...]}` by default.
With `--all`, they return a flat array `[...]`.

Errors are written to stderr as JSON: `{"error": "message"}`.
Exit code 0 = success, 1 = error.

## Commands

### Auth
- `bugsink auth login` — Interactive authentication (requires admin access for token creation)
- `bugsink auth status` — Check if credentials are configured locally
- `bugsink auth status --verify` — Verify credentials against the API
- `bugsink auth logout` — Remove stored credentials

### Teams
- `bugsink teams list` — List all teams
- `bugsink teams get <id>` — Get team by ID

### Projects
- `bugsink projects list [--team <id>]` — List projects, optionally filtered by team
- `bugsink projects get <id>` — Get project by ID
- `bugsink projects create --team <id> --name <name>` — Create a project

### Issues
- `bugsink issues list --project <id> [--sort digest_order|last_seen] [--order asc|desc]` — List issues for a project
- `bugsink issues get <id>` — Get issue by ID

### Events
- `bugsink events list --issue <id> [--order asc|desc]` — List events for an issue
- `bugsink events get <id>` — Get event by ID with full data payload
- `bugsink events stacktrace <id>` — Get event stacktrace as markdown

### Releases
- `bugsink releases list --project <id>` — List releases for a project
- `bugsink releases get <id>` — Get release by ID
- `bugsink releases create --project <id> --version <version>` — Create a release

### Schema Discovery
- `bugsink describe` — Fetch full OpenAPI schema (for runtime introspection)

## Typical Agent Workflow

```bash
# 1. Find the project
bugsink projects list --fields id,name

# 2. List recent issues
bugsink issues list --project 1 --sort last_seen --order desc

# 3. Get issue details
bugsink issues get 42

# 4. Get the latest event's stacktrace
bugsink events list --issue 42 --fields id
bugsink events stacktrace <event-id>
```
