---
author: you
branch: main
status: done
---
# Generar proyectos sin el aviso del symlink de skills

## Intent
`cargo generate` avisaba «Symbolic links not supported» en cada perfil porque los
templates incluían `.claude/skills` como symlink. Además, un directorio vacío
dejado por el generador hacía fallar a `scripts/check_agents.py` en vez de
recrear el enlace, así que el proyecto generado no exponía las skills.

## Scope
In: los 11 templates (symlink retirado), sus copias de `scripts/check_agents.py`,
`tools/verify_templates.py`, `tests/test_agent_guard.py`, `tests/test_templates.py`,
`README.md`, `CHANGELOG.md` y los `docs/SKILLS.md` de cada perfil.

Not: el contenido de las skills, el flujo v2 (`generator/compose.py`, que ya
ejecuta el validador) y la publicación en GitHub.

## Checklist
- [x] Retirar `.claude/skills` de los 11 templates. Prueba: `cargo generate --git` ya no imprime el aviso.
- [x] Crear el enlace en `check_agents.py` y reparar un directorio vacío. Prueba: test unitario nuevo y ejecución manual.
- [x] Ajustar verificador y tests al contrato «el enlace se crea tras generar». Prueba: suite completa.
- [x] Documentar el paso posterior en README y docs de skills.

## Evidence
- `python3 tools/verify_templates.py --generate` → `All requested checks passed`
  para los 11 perfiles, sin `Skipping` ni `Symbolic links` en el log.
- Simulación del comando del usuario con un repositorio git temporal:
  `cargo generate --git file:///tmp/repo-sim templates/desktop-egui --name my-desktop-app`
  → sin aviso; después `python3 scripts/check_agents.py` →
  `Agent readiness OK: 14 validated skills; .claude/skills (symlink)`.
- `python3 -m unittest discover -s tests -v` → 19 tests, `OK` (incluye
  `test_repairs_empty_claude_directory`).
- `python3 tools/check_docs.py` → `Documentation checks passed (30 files)`.

## Next
Commit y push a `JhonMA82/universal-rust-app-starter`; el comando original usa
`--git` contra GitHub, así que el arreglo surte efecto allí tras publicarlo.
