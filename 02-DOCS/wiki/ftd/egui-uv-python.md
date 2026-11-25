---
author: Juan José
branch: main
status: done
---
# uv como gestor de Python en las skills egui

## Intent
Las skills `egui-*` y las herramientas de Egui Visual Forge se instalaban con
`python3 -m pip install -r requirements.txt` contra el Python del sistema. Eso
mezclaba dependencias entre proyectos y no fijaba el intérprete. El objetivo es
que el perfil `desktop-egui` declare sus dependencias en un `pyproject.toml`
gestionado por `uv`: `uv sync` crea el entorno y `uv run` ejecuta los
comparadores sin depender de un `pip` global.

## Scope
In: `templates/desktop-egui/pyproject.toml` (nuevo), retirada de
`requirements.txt`, `cargo-generate.toml`, `.gitignore`, las cinco skills
`egui-*` que citan Python, `tools/validate_toolkit.py`, los copiados de
verificación, `README.md`, `AGENTS.md`, `docs/SKILLS.md`,
`docs/EGUI_VISUAL_FORGE.md`, `docs/VALIDATION.md` y `CHANGELOG.md`.

Not: el generador propio del repositorio (`generator/requirements.txt`), los
demás perfiles, la lógica de comparación de imágenes y las versiones pinneadas
de Pillow/NumPy/PyYAML.

## Checklist
- [x] Declarar Pillow, NumPy y PyYAML en un proyecto uv no empaquetable y
      retirar `requirements.txt`. Prueba: `uv sync` crea el entorno del perfil.
- [x] Ajustar `cargo-generate.toml`, `.gitignore` y el validador del toolkit al
      nuevo contrato. Prueba: `uv run --project ... python tools/validate_toolkit.py`.
- [x] Reescribir los comandos de las skills `egui-*` como `uv sync` + `uv run`.
      Prueba: las cinco skills siguen pasando el validador y no queda `python`
      suelto en sus comandos.
- [x] Actualizar la documentación del perfil y del repositorio. Prueba:
      `python3 tools/check_docs.py`.
- [x] Verificar sin romper la generación. Prueba: suite del repo y proyecto
      generado con `cargo generate` que conserva `pyproject.toml` y ejecuta
      `uv run python tools/validate_toolkit.py`.

## Evidence
- `uv sync` en `templates/desktop-egui` (uv 0.13.9, CPython 3.14.9) →
  `Resolved 6 packages`; instalados `numpy==2.5.5`, `pillow==12.3.0`,
  `pyyaml==6.0.4`.
- `uv run python tools/validate_toolkit.py` →
  `PASS: skill YAML, required files, local resource links, nonempty docs and Python syntax`.
- `uv run python -m unittest discover -s tests -v` → 13 tests, `OK`.
- `python3 tools/verify_templates.py` → `PASS offline: 11 profiles, manifests,
  workflows and mirrored agent skills`; `All requested checks passed`.
- `python3 tools/check_docs.py` → `Documentation checks passed (30 files)`.
- `python3 -m unittest discover -s tests -v` → 19 tests, `OK`.
- Generación real en `~/.cache/egui-uv-smoke`:
  `cargo generate --path templates/desktop-egui --name smoke-egui-uv` copió
  `pyproject.toml` y no copió `requirements.txt`, `.venv` ni `uv.lock`; después
  `uv sync`, `uv run python tools/validate_toolkit.py` → `PASS` y
  `uv run python -m unittest discover -s tests -q` → 13 tests, `OK`.
- La primera pasada de la suite copió el `.venv` local a `/tmp` y agotó la
  cuota del tmpfs. Se añadió `.venv`/`uv.lock` a los `ignore_patterns` de
  `tools/verify_templates.py`, `tests/test_templates.py` y
  `tests/test_composition.py`; la segunda pasada terminó en verde.
- `rg "requirements\.txt|python3? (tools/|-m pip)|python tools/"
  templates/desktop-egui` solo encuentra las formas `uv run python tools/...`
  documentadas.

## Next
Commit local en `main` (sin push). Publicar queda pendiente de autorización.
