---
author: Juan José
branch: main
status: done
---
# Aplicar el parche de auditoría domain-first

## Intent
Integrar `universal-rust-app-starter-auditoria-domain-first.zip` (4 commits de
Codex, entrega `deabd475` más la corrección `f052fef`) sobre `main` sin
reescribir su contenido.

## Scope
In: los 42 archivos del parche, aplicados como 4 commits con `git am`.
Not: los criterios de cierre que la auditoría deja abiertos (producto documental
nuevo sobre la sesión tipada, productos de datos e ingeniería desde archivos
propios, registro de menús y atajos en el shell, recetas compiladas, escenarios
de accesibilidad y matriz CI multiplataforma). Esas brechas son trabajo abierto,
no parte de esta aplicación.

## Checklist
- [x] `sha256sum -c SHA256SUMS`: 7 archivos OK.
- [x] `git am`: 4 parches aplicados sin conflictos (`fe65c9a`, `2a10706`,
  `d04f5e5`, `3cc6409`).
- [x] `python3 -m unittest discover -s tests`: 27 tests OK.
- [x] `python3 tools/check_docs.py`: 33 archivos OK.
- [x] `python3 tools/verify_templates.py`: 11 perfiles offline OK.
- [x] `tools/verify_domain_scenarios.py`: desktop, documents, data y
  engineering con estado `passed`.

## Evidence
`cargo-generate 0.26.0` instalado con `--locked`; `tomli-w 1.2.0` aportado por
`uv run --no-project --with-requirements generator/requirements.txt`.
Evidencia en `/tmp/domain-evidence.json`, fuera del árbol: commit `3cc6409`,
`source_dirty: false`, rustc `1.101.0-nightly`. Cada escenario generó su
workspace, extendió un dominio propio, pasó `cargo fmt --all`, `cargo test` por
paquete y ejecutó el binario por CLI o MCP. Tiempos de verificación: desktop
21.2 s, documents 2.9 s, data 17.8 s, engineering 11.8 s.
