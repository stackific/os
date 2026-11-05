# Prompt-trail entry template

Every task's PR adds one entry at `site/src/content/docs/prompt-trail/<NNN>-<slug>.md`, where `<NNN>` (three digits) is the task's number. `/s-build` writes it. A task whose `Base:` is `arch` or `fedora` puts its entry under `prompt-trail/arch/` or `prompt-trail/fedora/` instead, which the sidebar shows as the Arch and Fedora groups; numbers stay global.

Readers see the story of the step, and the task behind it stays one click away: `task:` in the frontmatter makes the site link the task file.

Expectations are the task's acceptance criteria, written before any building. AI models drift; the same prompt on a newer model may produce a different result. That is acceptable: the expectations, not the original diff, are the test. If a rebuild meets them it is valid; if not, the reader knows where it diverged.

The prompt is the task's ask plus every follow-up and review round that changed the result, consolidated and tidied into one request a reader can send as is. Conversational noise (typos, back-and-forth, dead ends) is left out; nothing that changed the result is.

````markdown
---
title: <title-without-the-incrementing-integer>
description: One line.
sidebar:
  order: 1
  label: <NNN> <title for sidebar>
pr: https://github.com/stackific/os/pull/<number>
task: dev/tasks/<NNN>-<slug>.md
---

(The site draws the `pr:` link and the `task:` link under the title. No link markup in the entry.)

## Goal
What this step was meant to produce.

## Context
What existed before and what this step implements, in two or three plain sentences. The task holds the details.

## Expectations
The task's acceptance criteria, copied as they are.

Must:
- observable outcome 1 (a file exists, a command prints X, a test passes)
- outcome 2

Must not:
- touch files outside the task's Changes list
- add a dependency the task does not name

## Prompt
The task's ask with every follow-up and review round that changed the result, consolidated into a single prompt. It is a wrapped code block titled "Prompt", so the docs show a copy button on it.

```text title="Prompt" wrap
prompt text
```

## Outcome
One coherent response to that prompt: what was built and how it meets the expectations.

### Screenshots
One image per distinct screen or state the expectations name (a boot menu, an installer screen, a desktop state). Each has alt text saying what it shows and an italic caption under it saying what to notice. Images live in `site/src/assets/prompt-trail/<NNN>/` and are made with `just site screenshot`; an entry under `arch/` or `fedora/` adds one more `../` to the path below. A task with no UI (packages, CI, tests, docs only; site pages are never screenshotted) has the single line "No UI." here instead.

![Limine menu with the OS first and Windows below it](../../../assets/prompt-trail/<NNN>/limine-menu.webp)

_The Limine menu after an alongside install._

## Review
Each review round, summarised from the task's record in `dev/reviews/<NNN>-<slug>.md`: what it found, and what was fixed or rejected and why. Do not mention the skill name here, i.e., `/ponytail-review`

## Manual changes
Anything done by hand, and why the agent didn't do it.

## Result
What now exists. Commands run to verify, with their output summarised. The docs pages added or changed.

## Deviations
Where the result differs from the task, and whether the task file was updated.
````
