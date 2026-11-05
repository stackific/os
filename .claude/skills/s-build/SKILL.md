---
name: s-build
description: Build one task from dev/tasks/ step by step, write its docs and prompt-trail entries, then run the ponytail review cycle until the task is commit ready, recording every round in dev/reviews/. Use when a task is ready to build, or to address review findings or PR comments on a task.
---

# /s-build

The goal is a commit-ready task: tests pass, the review finds nothing, the docs and prompt-trail entries are written, and a commit message is drafted per step. The operator commits.

## Steps

1. Read the task and `AGENTS.md`. Stop if the task's Done box is ticked.
2. Plan the steps, one commit each, and tell the operator the plan in a few lines.
3. Build each step:
   - start from the task's Inputs, which are already in the tracked tree;
   - write the code and config;
   - run the task's tests and the `just` checks the change touches (tests run without asking).
4. Docs entry: add or update the docs the change needs: user docs under `site/src/content/docs/user/`, developer notes in `docs/`.
5. Prompt-trail entry: write it from `entry-template.md` in this folder, with the task's number, in the folder its `Base:` names: `site/src/content/docs/prompt-trail/<NNN>-<slug>.md` for `both`, or `prompt-trail/arch/` or `prompt-trail/fedora/`. Make a screenshot with `just site screenshot` for each screen or state the criteria name.
6. Review cycle, on the whole change (code, tests, docs and entry):
   1. Run `/ponytail-review`.
   2. Record the round in `dev/reviews/<NNN>-<slug>.md`, written from `review-template.md` in this folder: one file per task, updated in place.
   3. If the review found nothing, set Status to `commit ready` and go to step 7.
   4. Otherwise fix every finding, rerun the tests, and record each finding's resolution and the test results in the round. Reject a finding only when fixing it would break the task's criteria or `AGENTS.md`, citing the line; a rejected finding counts as resolved.
   5. Review again from 6.1.
   - Cap: two fix rounds. If the review after the second fix round still finds anything, set Status to `stopped` and stop: tell the operator the cycle is not making progress, with the open findings and the record's path.
   - Act on security-guidance warnings too; a review does not replace tests.
7. Update the entry's Review and Result sections from the record.
8. Report:
   - what was built;
   - the commands run and their results;
   - the review record's path, its rounds and Status;
   - the files changed;
   - any manual steps, numbered;
   - a commit message per step (DCO sign-off and SSH signing are the operator's).

## Addressing findings

When called with review findings or PR comments, add them to the task's review record as a new round (name the source), then run the cycle from step 6.4 with a fresh two-round cap. Then update the entry: fold what changed the result into its Prompt, and add the round to Review and Result.

## Rules

- Build only what the task says. If the task is wrong or incomplete, stop and say so; the fix goes into the task file first.
- Never run git commands that change history; the operator does every git operation.
- Keep the change to the task's Changes list unless the operator agrees otherwise; anything else goes under Deviations in the entry. The task's review record is always part of the change.
