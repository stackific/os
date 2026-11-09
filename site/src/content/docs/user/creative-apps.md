---
title: Creative apps
description: Install PhotoCraft and twelve more apps for images, video, audio, documents and design with one click.
sidebar:
  order: 5
---

Stackific OS can install a set of creative apps for you in one step. They are made by their own developers and come straight from them, so they are not part of the Stackific OS download.

| App         | For                                           |
| ----------- | --------------------------------------------- |
| PhotoCraft  | Image editing: layers, masks, type, PSD files |
| VectorCraft | Vector illustration                           |
| FilmCraft   | Video editing, colour and sound               |
| LightCraft  | Photo library and raw development             |
| PdfCraft    | Reading, organising and protecting PDFs       |
| EffectCraft | Motion graphics and visual effects            |
| DesignCraft | Page layout and publishing                    |
| FigCraft    | Interface design and prototyping              |
| WordCraft   | Word processing and documents                 |
| SoundCraft  | Audio recording and editing                   |
| DeckCraft   | Presentations and slide decks                 |
| CADCraft    | Computer-aided design                         |
| GridCraft   | Spreadsheets and data                         |

Most of them are early versions; PhotoCraft is the most complete.

## Install them

1. Install Stackific OS first: the apps need about 5 GiB of free space, which the live session does not have.
2. Open the app launcher and start **Install creative apps**.
3. Type your password when asked.
4. Wait until the window says "Done". This downloads about 1.3 GiB, so it takes a while on a slow connection.

The apps then appear in the app launcher. Before anything is installed, each download is checked against its developers' signature and against the version Stackific OS expects; if one check fails, nothing is installed and the window says which file failed.

## Remove them

Each app is an ordinary package. On the Arch Linux images it is named like `ai.storyteller.photocraft`: remove it with `sudo pacman -R ai.storyteller.photocraft`. On the Fedora Remix images it is named like `photocraft`: remove it with `sudo dnf remove photocraft`.

## Updates

Starting **Install creative apps** again installs the versions this Stackific OS release expects. Newer versions arrive with Stackific OS updates.
