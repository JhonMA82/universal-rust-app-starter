---
author: Juan José
branch: main
status: in-progress
---
# Consistencia documental de la v2

## Intent
Cerrar los desajustes entre la documentación del candidato v2 (commit
`0472aa4`) y el estado real del repositorio. Ninguna afirmación debe depender
de artefactos ausentes, y el contrato de agentes debe cubrir la mitad modular.

## Scope
In:
- `docs/V2_REPORT.md`: cambiar la afirmación de evidencias adjuntas por los
  comandos reproducibles y los artefactos de CI.
- `CHANGELOG.md`: registrar el candidato modular v2 en `Unreleased`.
- `AGENTS.md`: documentar `generator/`, `modules/`, `presets/`, `examples/` y
  `tools/verify_composed.py` en la verificación y las reglas.
- `docs/VALIDATION.md`: corregir el alcance del título.
- `docs/MODULE_CATALOG.md`: alinear los pins de dependencias con los manifiestos.
- `templates/desktop-egui/docs/EGUI_VISUAL_FORGE.md`: describir el symlink de
  Claude en lugar de un mirror.
- `docs/SOURCES.md`: acotar rmcp 3.11 a la referencia modular.

Not:
- Ejecutar la suite compuesta: requiere `cargo-generate`, no instalado aquí.
- Cambiar código, manifiestos, presets ni workflows.
- Versionar evidencias ni crear `evidence/`.

## Checklist
- [x] Escribir el documento FTD; prueba: este fichero.
- [x] Corregir V2_REPORT; prueba: sin «acompañan al parche» y el enlace a
      `VALIDATION.md` resuelve.
- [x] Registrar la v2 en CHANGELOG; prueba: `## Unreleased` menciona
      `compose.py`, 17 módulos versionados y presets.
- [x] Actualizar AGENTS.md; prueba: `verify_composed.py`, `generator/compose.py`
      y `module.json` presentes.
- [x] Corregir VALIDATION, MODULE_CATALOG, forge y SOURCES; prueba: script de
      consistencia con 0 hallazgos y 17/17 pins alineados.
- [x] Verificar estáticamente; prueba: `verify_templates.py` PASS y
      `unittest discover` 15 tests OK.
- [x] Actualizar la versión del README; prueba: el título dice
      `v2.0.0 candidate` y no queda `v1.0.0`.

## Evidence
- `python3 tools/verify_templates.py` → `PASS offline: 11 profiles, manifests,
  workflows and mirrored agent skills` / `All requested checks passed`, exit 0.
- `python3 -m unittest discover -s tests -v` → `Ran 15 tests ... OK`, exit 0.
- Script de consistencia → `ISSUES: 0`, `catalog rows: 17 modules: 17`
  (ids, targets, deps pinneadas, docs, CHANGELOG, AGENTS, título, forge,
  SOURCES y enlaces internos).
- Revisión fresh-context → 2 defectos de atribución: `validate-templates.yml`
  no produce ni sube artefactos, y el workflow modular no empaqueta. Corregidos
  en `V2_REPORT` (productor correcto y paquete manual del módulo `distribution`)
  y en `CHANGELOG` (`generate, test and lint`). Revalidación → `ISSUES: 0`,
  `verify_templates.py` PASS y 15 tests OK.
- Límite: la suite compuesta no se ejecutó; `cargo generate` no está instalado.
- `README.md:1` → `# Universal Rust App Starter · v2.0.0 candidate`;
  `grep v1.0.0 README.md` no devuelve nada.

## Next
Sin siguiente paso; el hook de cierre de rsc marcará el estado.
