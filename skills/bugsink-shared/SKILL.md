---
name: bugsink-shared
description: Runtime contract for the bugsink CLI. Covers authentication, output format, error handling, and invocation patterns. Use as foundation before any other bugsink skill.
compatibility: Requires bugsink binary installed and network access to a Bugsink instance.
---

# bugsink-shared

Foundation skill for the bugsink CLI. Read this before using any other bugsink skill.

## Authentication

Agents should use environment variables:

```bash
export BUGSINK_URL="https://your-bugsink-instance.com"
export BUGSINK_TOKEN="your-api-token"
```

Or pass per-command:

```bash
bugsink --url "$BUGSINK_URL" --token "$BUGSINK_TOKEN" <command>
```

To check if credentials are configured:

```bash
bugsink auth status --json
```

To verify credentials work against the API:

```bash
bugsink auth status --verify --json
```

## Invocation Pattern

Always use `--json` to ensure compact, parseable JSON output:

```bash
bugsink --json <command>
```

Use `--fields` to reduce output to only needed fields (saves tokens):

```bash
bugsink --json --fields id,name projects list
```

Use `--all` to fetch all pages of paginated results:

```bash
bugsink --json --all teams list
```

## Output Format

- **Success**: JSON to stdout, exit code 0
- **Error**: JSON to stderr (`{"error": "message"}`), exit code 1

List commands without `--all` return a paginated envelope:

```json
{"next": "...", "previous": null, "results": [...]}
```

With `--all`, they return a flat array:

```json
[{"id": 1, ...}, {"id": 2, ...}]
```

## Error Handling

Always check exit code. On failure, parse stderr for the error message:

```bash
result=$(bugsink --json teams get 999 2>err.tmp) || {
  error=$(cat err.tmp)
  # handle error
}
```

## Available Resources

- Teams: `bugsink teams list|get`
- Projects: `bugsink projects list|get|create`
- Issues: `bugsink issues list|get`
- Events: `bugsink events list|get|stacktrace`
- Releases: `bugsink releases list|get|create`
- Schema: `bugsink describe` (full OpenAPI spec)
