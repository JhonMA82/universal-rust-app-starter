---
type: spec
title: Usability layer — Domain-first, capability-first starter
slug: usability-layer
status: draft
date: 2026-11-25
branch: main
---

# Spec — Usability layer

## Problem & why

The starter already ships modular infrastructure (profiles, presets, modules, crates, templates, verification).
But selecting and using that infrastructure still requires the developer to investigate which modules exist,
how they relate, how to configure them and how to consume the resulting capabilities.
The existence of infrastructure does not guarantee it is easy, immediate and productive to use.
The result is that each new project re-solves transversal problems the starter claims to have solved.

## Cost of not building it

Every generated project spends its first phase reconstructing infrastructure instead of domain logic;
agents duplicate components because discovery is weak; documentation drifts from manifests.

## The cheapest alternative

Keep the current generator and document modules better. Insufficient: documentation alone cannot
integrate crates, resolve dependencies, or expose a capability inventory derived from composition.

## Goals

- A user can create a project by choosing an application type and capabilities, not crates.
- Selected capabilities arrive installed, configured, integrated, accessible, documented and tested.
- Generated projects are autonomous and agent-discoverable: capability inventory, component catalog,
  extension guide, verification commands, skills matching the composition.
- Documentation matches manifests and composition.

## Non-goals / out of scope

- New specialized engines (CAD solver, scientific solver, acoustic engine, generic data-analysis engine).
- A permanent runtime/framework controlling generated projects.
- GUI generator (CLI/TUI generation is acceptable).
- Replacing the existing modular generator with a different one.
- Changing the eleven original profiles or their contracts without justification and migration.

## Users & context

- Human developers starting a real application.
- Programming agents with no prior knowledge of the starter, operating on a generated project.

## What the user can do

A developer (or agent) runs the generator, selects a profile, preset and capabilities, previews the
composition, generates, and verifies — without knowing internal crate names. They then add domain
entities, commands, views and tests using documented extension points.

## What result they get

A standalone project whose `docs/COMPOSITION.md`, `AGENTS.md`, `README.md`, `generation-report.json`
and `.agents/skills/` reflect the actual composition, and whose verification commands pass.

## What it must respect

- Existing profiles, presets, modules and their contracts (constitution principles 2-4).
- Autonomy: no unpinned Git deps, no external paths, no escaping symlinks.
- Licensing: MIT OR Apache-2.0 for own code.
- Decision log and constitution already ratified.

## How it proves it

- `python3 tools/verify_templates.py` and `python3 -m unittest discover -s tests` pass.
- Generated projects pass their own verification matrix.
- `scripts/check_agents.py` passes in generated projects.
- `rsc spec gate` passes on this spec.

## Behaviour

- Main: profile + preset + capabilities → resolver integrates modules → generated project with
  inventory and skills → verification passes → developer adds domain code.
- Edge: invalid capability combination is rejected before generation with a clear message.
- Error: missing environment tool (rustfmt, cargo-generate) is reported before generation.

## Acceptance criteria

- Given a developer selecting `desktop-egui` + `application` preset, When generation completes,
  Then `docs/COMPOSITION.md` lists the installed modules and entrypoints.
- Given a generated project, When `scripts/check_agents.py` runs, Then it passes and skills match
  the composition.
- Given a generated project, When its `AGENTS.md` is read, Then it names the architecture, domain
  location, infrastructure, component catalog, extension guide, verification tools and skills.
- Given a capability not selected, When its docs are inspected, Then it is not announced as available.
- Given the verification matrix, When run on the starter, Then all checks pass.

## Points to clarify

- **decisión diferida** — full interactive TUI generator: deferred; CLI + declarative composition
  remains canonical. Reason: avoid a second generator engine (constitution principle: single
  mechanism).
- **decisión diferida** — product presets (Document/Data/Engineering/Creative/Scientific Workbench):
  evaluate against existing presets; reuse where possible.
- **decisión diferida** — external UI Kit adoption (Craft Libs): requires license audit; record
  decision before adoption.
- **suposición tomada** — first implementation increment focuses on agent-ready generated projects
  (capability inventory + composition-aware AGENTS.md + skills by capability), because it is the
  highest-leverage, verifiable slice and unblocks agent scenario E.
  *Base:* encargo Phases 7-8. *Riesgo:* broader usability items remain for later increments.
- **área no formulable** — how much of the 24 Definition-of-Done items are already satisfied by V2;
  needs an audit pass before claiming completion.

## Revisions

- 2026-11-25: initial draft from encargo.

## Clarifications

- 2026-11-25: Validated suposición tomada — first increment focuses on agent-ready generated projects. Holds: highest-leverage verifiable slice.
- 2026-11-25: Left deferred — interactive TUI generator, product presets, external UI Kit adoption.
- 2026-11-25: Área no formulable noted — audit of 24 DoD items vs V2; graduates when audit runs.
- 2026-11-25: Gap checklist walked; no new blocking questions. Composition-derived inventory is the approved reference for "what result they get".
