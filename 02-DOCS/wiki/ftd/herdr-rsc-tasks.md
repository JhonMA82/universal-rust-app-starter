---
author: you
branch: main
status: in-progress
---
# Herdr rsc task panel

## Intent
Surface the ASDD and FTD work of every space inside the Herdr window, so the next
task and the decisions that block it are visible without opening `02-DOCS`
by hand.

## Scope
In:
- Add the Shepherdr layout from `eliasstravik/shepherdr` to `~/.config/herdr-app`.
- A new Herdr mod, `mods/rsc-tasks`, that scans each space's repo for
  `02-DOCS/wiki/ftd/*.md` and `02-DOCS/wiki/sdd/{plans,progress}/*.md`.
- Rows show checklist progress, status and blocking decisions; a click opens the
  document inline.
- Dock the panel in the current `gentle-herdr` layout.

Not:
- Starting or editing ASDD/FTD artifacts from the panel.
- Replacing Shepherdr's task engine; Shepherdr's own tasks need Claude Code or
  Codex, neither installed here.
- Remote-space file watching (remote rows refresh on the timer only).

## Checklist
- [x] Add the Shepherdr layout; proof: `./install.sh --no-build --no-hooks
      --keys keep --yes` reported 813 new files, `shepherdr doctor` said
      `✓ whole`, and switching to `shepherdr` plus `herdr-app shot` showed its
      sidebar, chat and Tasks panel.
- [x] Write the parser; proof: `node /tmp/rsc-parse-test.mjs` printed
      `FTD {status: 'in-progress', done: 1, total: 2, next: 'Ship the parser.'}`
      and `ASDD {total: 3, done: 1, current: T002, blocking: 1, status: 'needs you'}`,
      then `PARSER OK`.
- [x] Write the panel; proof: `herdr-app check ~/.config/herdr-app/mods/rsc-tasks`
      printed `check passed` and `herdr-app status` kept `"errors": []`.
- [x] Prove it live; proof: the docked panel showed this document as
      `FTD Herdr rsc task panel 0/4`; a temporary space at `/tmp/asdd-fixture`
      added `ASDD Fixture plan 1/3` with `1 blocking`, and closing it returned
      the panel to one item.

## Evidence
- Shepherdr 3.1.44 installed into `/home/you` from the checkout at
  `~/dev/shepherdr`, with the web build, Claude/Codex hooks, key overrides and
  the task browser download disabled. `shepherdr doctor` reports the install
  whole; it also reports bun absent (voice, web) and no web service. The layout
  is `shepherdr`; the window was switched back to `gentle-herdr` afterwards.
- Shepherdr's task engine needs Claude Code or Codex; neither is on the PATH,
  so its Tasks panel stays empty. Its layout still loads and is available with
  `Ctrl+Shift+L`.
- `rsc-tasks` lives at `~/.config/herdr-app/mods/rsc-tasks` (`main.js`,
  `parse.js`, `mod.json`) and is docked at the right of `gentle-herdr` through
  `layouts/gentle-herdr.json` node `rsc-tasks` (360 px).
- `Ctrl+Shift+A` is bound to `mod.open:rsc-tasks`, so the panel also opens as a
  dialog over the Shepherdr layout; pressing it again closes that copy.
- No live ASDD artifact exists in this repo yet; the ASDD proof used the
  `/tmp/asdd-fixture` space, removed after the screenshot. The parser fixture
  covers plan tasks, progress statuses, `blocking` and `review-after`.

## Next
Nothing pending. Re-read the panel after the next restart; it rescans every 15 s
and on the ↻ button.
