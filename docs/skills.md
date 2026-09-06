# Ambient Context agent skills

The MCP server gives an agent twenty tools. The skills tell it when to use
them, how much of the record to read, and what to leave alone. Ambient
Context ships five and installs them from Settings > Agent skills, into the
two directories most agents read:

| Directory | Read by |
|---|---|
| `~/.claude/skills/` | Claude Code |
| `~/.agents/skills/` | Cursor, Codex CLI, Zed, GitHub Copilot, Gemini CLI, Goose and OpenCode |

Each skill is a folder holding a `SKILL.md` in the format at
[agentskills.io](https://agentskills.io/specification). The source is the
`skills/` folder of this repository, so anyone who prefers their own tooling
can run:

```
npx skills add dragthelake/ambient-context
```

## Install, update, remove

**Install** writes the five folders into both directories and records what
it wrote in `skills-install.json` in the app's data directory. The Overview
shows an **Install agent skills** button until this has happened once.

**Update** appears when a new version of the app carries a changed skill.
It overwrites only files the app wrote. A `SKILL.md` you have edited is left
alone and named in the panel, with a **Replace my edits** button if you want
the shipped version back.

**Remove** deletes the files the app wrote and leaves any you edited. The
Overview does not ask again after a remove.

Each install, update and remove is written to that day's ledger as
`install_skills`, `update_skills` or `remove_skills`, with the paths.

## The skills

### `ambient-context`

The core loop, and the one the other four defer to. Triggers when the user
refers to what they were working on, their day or week, their record, or
Ambient Context. Explains the record (three raw files per day, summary,
knowledge base, ledger), which tool answers which question, restraint (read
the narrowest thing, quote little, zero is a normal result), privacy (the
record stays on this machine), and what to say when the server is not
registered or the app is closed. Reads: `list_days`, `read_summary`,
`read_day`, `read_kb`, `search_record`, `read_ledger`, `capture_status`.
Never writes unless asked.

### `ambient-context-catch-me-up`

Triggers on "what was I doing", "where did I get to", "catch me up". Reads
today's summary (and yesterday's before 10:00), the knowledge base threads
and commitments for anything unfinished, and the raw record only for the
last hour. Replies with where you were, the open threads, and one place to
pick up.

### `ambient-context-standup`

Triggers on "standup", "daily update", "status for the channel". Reads the
last working day's summary and commitments, and today's record so far.
Writes yesterday, today and blocked in your voice, ready to paste. Names
only work the record shows.

### `ambient-context-weekly-review`

Triggers on "weekly review", "week note", "what happened this week". Reads
the last seven summaries and the latest threads file. Writes a week note:
three lines, by project, stalled, patterns, next week.

### `ambient-context-tune-rules`

Triggers on "stop capturing X", "that keeps landing in the wrong file",
"tidy my rules". Reads the current rules and the evidence in the record,
proposes the smallest change as a diff, and applies it with `add_rule`,
`update_rule` or `remove_rule` only after you say yes. Needs the app open.

## Editing a skill

The installed files are yours. Edit `~/.agents/skills/<name>/SKILL.md` or
the Claude Code copy and the app will not overwrite it. The panel shows
`edited` beside the skill, and Update skips it. To go back to the shipped
version, press **Replace my edits**.
