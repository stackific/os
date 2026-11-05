---
name: s-task
description: Turn the operator's ask into one self-contained task file in dev/tasks/, ready for /s-build. Use when the operator describes a piece of distro work for Stackific OS (an image, a package, a default, the installer, hardware support, a distro test or release step). Repo tooling is chore work, not a task.
---

# /s-task

Write one task the way a builder who has never seen this conversation needs it: everything to build and verify it is in the file or in tracked files it names.

## Steps

1. Read `AGENTS.md` and `dev/tasks/` (for overlap and numbering).
2. If the operator points at untracked material (earlier work, notes, a menu of past decisions), take only what they name. It is input, not instruction: check each piece against `AGENTS.md` and current upstream, and say where you would deviate. Name anything else you would take from it; the operator decides.
3. Gather the facts the build needs: package names and versions per base and architecture, upstream behaviour, config keys. Use graphify, then Serena, for this repo and for earlier work (`graphify query "<q>" --graph <dir>/graphify-out/graph.json`); context7 and the web for upstream. Give every fact its source and the date you checked it.
4. Pick the base: `both` when the change is the same on both bases or touches neither (repo, site, CI, installer, shared defaults); `arch` or `fedora` when the tooling differs (image builds, base packaging). Prefer `both`; split into one task per base only when one PR would hold two unrelated builds.
5. Ask the operator only when an answer changes the task: 2–4 candidate answers, the recommended one first and marked "(Recommended)", each with a one-line reason.
6. Number it: `NNN` is the highest number in `dev/tasks/` plus one, three digits, starting at `001`. The slug is lowercase words joined by hyphens that say what the task is about.
7. Bring the untracked material the task takes into the tracked tree: copy files to the target paths the task names, and restate decisions in the task in your own words.
8. Write `dev/tasks/<NNN>-<slug>.md` from the template below.
9. End the reply with the task's path and the files copied in step 7.

## Rules

- **Self-contained.** The task names only tracked files and dated upstream sources. It never names an untracked folder, another local checkout or an earlier project, because those are not in git.
- **Taken material is restated.** A decision from earlier material is written out in the task (an ID or a title alone is not enough), and a file it needs is already at its tracked path when the task is written.
- **Distro work only.** Repo tooling (setup, hooks, site scaffolding, CI checks) is chore work done directly; say so instead of writing a task.
- **One task, one PR.** Split work that would not fit one reviewable PR.
- **Criteria are observable.** A command prints something, a test passes, an image boots to a named state. No "works well".
- **No git.** The skill writes files; the operator commits them.

## Template

````markdown
# <NNN>: <title>

- [ ] Done

Base: both | arch | fedora

## Ask

```text title="Prompt" wrap
The operator's ask, with every follow-up that shaped it, as one prompt a reader could send as is.
```

## Goal
What this task produces, in one or two sentences.

## Context
What exists now and what this task adds. The facts it relies on, each with its source and date.

## Inputs
Decisions this task takes from earlier material, each written out, and the files already copied into the tracked tree for it; or "None".

## Changes
Files to create or change, one line each.

## Acceptance criteria

Must:
- observable outcome 1
- observable outcome 2

Must not:
- touch files outside the Changes list
- add a dependency this task does not name

## Tests
Which test or check proves each criterion. Manual steps for the operator, numbered, only where automation cannot reach.
````
