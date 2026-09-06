---
name: ambient-context-tune-rules
description: Use when the user says something should not be captured, is landing in the wrong file, keeps showing up in summaries, or asks to tidy or review their Ambient Context rules. Proposes rule changes as a diff and applies them only after the user agrees.
license: Apache-2.0
metadata:
  tools: "capture_status list_rules search_record read_day add_rule update_rule remove_rule"
---

# Tune rules

Follow the `ambient-context` skill's conventions. This skill writes, so the app must be open; `capture_status` returning `not_running` means stop and say so.

A rule has a target (an `app`, a `website` or a window `title`) and an action: `exclude` (never keep), `headings_only` (keep the block heading, drop the body), `full` (keep everything, overriding a broader rule) or `route_messages` (send the block to the `messages` file). Every rule change is logged in the ledger with this client's name.

## Steps

1. `list_rules` first. Read the built-in rules as well as the user's; many requests are already covered, and built-ins cannot be weakened.
2. Find the evidence. `search_record` for the app, site or phrase the user named, or `read_day` on the window they mean. Quote the heading, not the body.
3. Draft the smallest change that does what they asked: one rule where one will do. Prefer editing an existing rule over adding a near-duplicate.
4. Show the change as a diff: the rule before (or "none") and the rule after, with one line on what it will stop or move from the next snapshot on. Say that text already recorded is not changed.
5. Wait for a yes. Then apply with `add_rule`, `update_rule` or `remove_rule`, one call per rule.
6. Confirm in one line what was applied and that it is in today's ledger.

## Do not

- Apply anything before the user agrees, even a removal they asked for.
- Add a rule from a single occurrence without saying it was one occurrence.
- Change the prompt or config here; that is a different request.
