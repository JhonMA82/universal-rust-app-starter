---
author: you
branch: main
status: done
---
# Aplicar el parche v2-cierre

## Intent
Integrar el parche `universal-rust-app-starter-v2-cierre.patch` (4 commits de
Codex) sobre `main` sin reescribir su contenido.

## Scope
In: los 37 archivos del parche, como cuatro commits `git am`.
Not: la matriz completa de release (builds compuestos, WGPU, navegador).

## Checklist
- [x] `git am` aplica los 4 parches sin conflictos.
- [x] `python3 -m unittest discover -s tests -v`: 18 tests OK.
- [x] `python3 tools/check_docs.py`: 30 archivos OK.
- [x] `python3 tools/verify_templates.py`: 11 perfiles offline OK.
- [x] `py_compile` de los 5 scripts Python tocados: OK.

## Evidence
Commits `f21c49a`, `4c8492f`, `0798052`, `416d922` sobre `2ffe676`. Árbol
limpio tras la aplicación. La verificación compuesta y de navegador del propio
parche queda pendiente de ejecutarse cuando se pida la matriz de release.
