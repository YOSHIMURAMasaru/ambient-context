---
name: ambient-context
description: Use when the user refers to what they were working on, asks to catch up, mentions their day, week, record or Ambient Context, or when a task needs to know what happened recently on this Mac. Reads the Ambient Context record through its MCP tools and says how much of it to read.
license: Apache-2.0
metadata:
  tools: "capture_status list_days read_summary read_day read_kb search_record read_ledger summarise_day ingest_day add_rule update_rule remove_rule set_prompt set_config"
---

# Ambient Context

Ambient Context is a macOS menu bar app that records what its user works on. Every few seconds it reads the text of the focused window and appends it to a plain-text record, one folder per day, on the user's own disk. An agent the user runs later turns each day into a summary and a small knowledge base. The app is also an MCP server named `ambient-context`, and that server is how you reach the record.

## The record

For each day there is:

- **Three raw files.** `apps` (everything not routed elsewhere), `websites` (a table of pages visited with dwell time) and `messages` (chat and mail). Blocks are headed `## HH:MM–HH:MM · App · Window title`. A block body holds only lines new to that day, so a bare heading means the user was there, looking at things already recorded. The text is an accessibility-tree scrape, not prose: good for what was on screen, silent about the rest. `[redacted]` marks a scrubbed secret.
- **A summary.** One distilled account of the day, written by the user's agent, with time-range citations.
- **A knowledge base.** Six files (people, commitments, threads, products, issues, reading), every line cited.
- **A ledger.** Every model action and every configuration change, with who made it. Your writes appear here under this client's name.

## Which tool

| You want | Call |
|---|---|
| What exists | `list_days` |
| What a day was about | `read_summary` first; `read_kb` for people, commitments and threads |
| Detail on a window of time | `read_day` with `from` and `to`, and `file` when it is not `apps` |
| When something happened | `search_record` |
| Whether recording is on and what is running | `capture_status`, and only when that is the question |
| What changed and who changed it | `read_ledger` |

A day with no summary can be built with `summarise_day` and `ingest_day`. Both take minutes and run the user's own agent on their machine, so say that you are starting one, and do not start a second while one runs.

## Restraint

- Read the narrowest thing that answers the question: a summary before a day file, a time window before a whole day, a search before a read.
- Quote a few lines when they carry the answer. Never paste a day file, a summary or a block wholesale into a reply.
- A record that does not contain the thing is a normal result. Say so plainly and do not fill the gap with a guess.
- Reading needs nothing running. Writing (capture, rules, the prompt, config) needs the app open, and every write is logged with your name on it. After a write, say what you changed in one line.
- Do not call `add_rule`, `update_rule`, `remove_rule`, `set_prompt` or `set_config` unless the user asked for that change. The `ambient-context-tune-rules` skill handles rule changes and asks first.

## Privacy

The record can hold messages, page contents and documents the user never meant to share. It stays on this machine. Do not send it to another service, write it into files in a repository, paste it into an issue or a commit, or summarise it somewhere other people read, without asking first and naming what would leave.

## When the server is not there

If the tools are missing, the server is not registered: ask the user to open Ambient Context, Settings, MCP server. If a write returns `not_running`, the app is closed: reads still work and writes will once it is open. Say which of the two it is and stop. Do not guess at the record.
