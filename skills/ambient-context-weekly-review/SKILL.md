---
name: ambient-context-weekly-review
description: Use when the user asks for a weekly review, a week note, what happened this week, or what they spent the week on. Reads the last seven Ambient Context summaries and writes the review.
license: Apache-2.0
metadata:
  tools: "list_days read_summary read_kb"
---

# Weekly review

Follow the `ambient-context` skill's conventions.

## Steps

1. `list_days`. Take the last seven calendar days that have a summary. If fewer than three do, say so and offer to run the review on what exists.
2. `read_summary` for each, oldest first. Seven summaries are the whole input; do not open day files unless a summary names a thread you cannot place.
3. `read_kb` with `file` set to `threads.md` for the most recent day, to see which threads are still open.

## Reply

A week note, under 400 words, with these headings:

- **The week in three lines.** What it was mostly about.
- **By project.** One paragraph each, largest first. What moved, what shipped, what stalled.
- **Stalled.** Threads that appear early in the week and never resolve. Name the last day each was touched.
- **Patterns.** Apps or sites that took more time than the work seemed to need; days that went to meetings or messages rather than the plan. State what the record shows and leave the judgement to the user.
- **Next week.** Up to three suggestions, each traceable to a thread above.

Cite days by date. Do not quote message text.
