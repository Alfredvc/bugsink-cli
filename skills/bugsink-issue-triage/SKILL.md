---
name: bugsink-issue-triage
description: Triage errors in Bugsink. Find recent issues, inspect event details, and read stacktraces. Use when investigating bugs, diagnosing errors, or reviewing error trends for a project.
compatibility: Requires bugsink binary installed and configured (see bugsink-shared skill).
---

# bugsink-issue-triage

Workflow for investigating errors tracked in Bugsink.

## Step 1: Find the project

```bash
bugsink --json --all --fields id,name projects list
```

If you know the team, filter by it:

```bash
bugsink --json --fields id,name projects list --team <team-id>
```

## Step 2: List recent issues

Sort by `last_seen` descending to see the most recently active issues:

```bash
bugsink --json issues list --project <project-id> --sort last_seen --order desc
```

Or sort by `digest_order` for the default grouping:

```bash
bugsink --json issues list --project <project-id> --sort digest_order --order desc
```

## Step 3: Get issue details

```bash
bugsink --json issues get <issue-id>
```

## Step 4: List events for the issue

Events are individual occurrences. Most recent first:

```bash
bugsink --json events list --issue <issue-id> --order desc
```

To get just event IDs:

```bash
bugsink --json --fields id events list --issue <issue-id> --order desc
```

## Step 5: Get the stacktrace

This returns markdown-formatted stacktrace, not JSON:

```bash
bugsink events stacktrace <event-id>
```

Do NOT use `--json` with `stacktrace` — it outputs raw markdown.

## Step 6: Get full event data

For the complete event payload (including context, tags, environment):

```bash
bugsink --json events get <event-id>
```

## Tips

- Start with the most recent event (`--order desc`) — it has the freshest context.
- Use `--fields` liberally to reduce output size.
- The stacktrace command is the fastest way to understand what went wrong.
- If there are many events, look at a few to see if the pattern is consistent or if there are different failure modes.
