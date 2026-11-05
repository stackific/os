# Review record template

`/s-build` keeps one review record per task at `dev/reviews/<NNN>-<slug>.md`, with the task's number and slug. Each review appends a round and updates the Status line; earlier rounds stay as they were. The record shows how the task became commit ready, or why the cycle stopped.

Finding statuses: `open` (not yet resolved), `fixed`, `rejected` (fixing it would break the task's criteria or `AGENTS.md`; the resolution cites the line). Groups and the verdict line are the review's own (Must fix, Should fix, Nice to have). A round whose review found nothing has its verdict line and no table.

````markdown
# <NNN>: <title>: review

Task: dev/tasks/<NNN>-<slug>.md
Status: open | commit ready | stopped (not making progress)

## Round 1
Reviewed: what the review covered (the task's uncommitted change, or the PR comments it answers).
Verdict: the review's verdict line, as given.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | One-line statement of the problem | `path:line` | fixed | What changed |
| 2 | Should fix | One-line statement of the problem | `path:line` | rejected | Why, citing the task criterion or `AGENTS.md` line |

Tests after the fixes: the commands run and their results.

## Round 2
Reviewed: …
Verdict: Looks good. Ship.
````
