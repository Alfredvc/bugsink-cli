---
name: bugsink-project-overview
description: Get an overview of a Bugsink instance — teams, projects, releases, and issue counts. Use when onboarding to a project, auditing error tracking setup, or getting a high-level status report.
compatibility: Requires bugsink binary installed and configured (see bugsink-shared skill).
---

# bugsink-project-overview

Workflow for getting a high-level view of a Bugsink instance.

## List all teams

```bash
bugsink --json --all teams list
```

## List projects for a team

```bash
bugsink --json --all projects list --team <team-id>
```

Or list all projects across teams:

```bash
bugsink --json --all --fields id,name projects list
```

## Check releases for a project

```bash
bugsink --json releases list --project <project-id>
```

## Check issue volume for a project

```bash
bugsink --json issues list --project <project-id> --sort last_seen --order desc
```

## Create a new project

```bash
bugsink --json projects create --team <team-id> --name "My New Project"
```

## Create a release

```bash
bugsink --json releases create --project <project-id> --version "1.2.0"
```

## Discover the full API

For the complete OpenAPI schema (useful for understanding all available fields and endpoints):

```bash
bugsink --json describe
```
