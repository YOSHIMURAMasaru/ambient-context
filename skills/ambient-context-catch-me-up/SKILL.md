---
name: ambient-context-catch-me-up
description: Use when the user asks what they were doing, where they got to, what is still open, or to be caught up after a break. Reads today's Ambient Context summary and record and briefs them.
license: Apache-2.0
metadata:
  tools: "list_days read_summary read_day read_kb search_record"
---

# Catch me up

Follow the `ambient-context` skill's conventions: narrowest read first, quote little, zero is a normal result.

## Steps

1. Call `list_days` once. Note whether today and yesterday have a summary.
2. Read today's summary with `read_summary`. Before 10:00 local time, read yesterday's as well; the morning question is usually about yesterday.
3. If a day has no summary, read the headings of its record instead: `read_day` with `file` set to `apps`, and treat the `##` headings as the timeline. Read block bodies only for the last hour before the user stopped.
4. For anything the summary calls unfinished, open, blocked or waiting, check `read_kb` with `file` set to `threads.md` and then `commitments.md` for the citation, then `read_day` on that time window if the summary is not enough.
5. If the user names a thing the summary does not mention, `search_record` for it before saying it is not there.

## Reply

Under 200 words unless asked for more, in this order:

- **Where you were.** The last thing they were doing, with the app and the time.
- **Open threads.** One line each: what, with whom, what it is waiting on.
- **Pick up here.** One suggestion, drawn from the record, not invented.

Cite times as the record does (`09:14–09:41`). Do not list every block of the day.
