---
name: ambient-context-standup
description: Use when the user asks for a standup, daily update, status for a channel, or what to say they did. Drafts it from yesterday's Ambient Context summary and today's record so far, in the user's voice.
license: Apache-2.0
metadata:
  tools: "list_days read_summary read_day read_kb"
---

# Standup

Follow the `ambient-context` skill's conventions.

## Steps

1. `list_days`, then `read_summary` for the most recent working day before today. Skip weekend days unless the record shows work on them.
2. `read_summary` for today if one exists; otherwise `read_day` with `file` set to `apps` for today and read the headings only.
3. `read_kb` with `file` set to `commitments.md` for that day, for anything promised to someone.

## Reply

Three short sections, first person, plain sentences the user could paste into Slack or Linear without editing:

- **Yesterday.** What moved, by project. Outcomes over activity: "shipped the update slot state machine", not "worked on update.rs for four hours".
- **Today.** What the record shows them already on, plus any commitment due.
- **Blocked.** Only if the record shows a wait on someone or something. Leave the section out when there is nothing.

Name only work the record shows. If the day is thin, say so rather than pad it. No emoji, no nested bullets, no times unless they matter.
