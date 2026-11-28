---
author: Juan José
branch: main
status: done
---
# Producto documental de inspección sobre la sesión tipada

## Intent
Cerrar el primer criterio abierto de la auditoría domain-first: un producto
documental nuevo registra su formato y reutiliza apertura, guardado, autosave y
recuperación sin replicar la infraestructura del workbench de notas.

## Scope
In: `Recovery` y `Document::{autosave, recover, discard_recovery}` en
`starter-documents`; `inspection-workbench` con formato propio, registro de
operaciones y vista nativa; módulo, capability y producto; escenario de
verificación y documentación.
Not: los demás criterios del acta (datos e ingeniería desde archivos propios,
menús y atajos en el shell, recetas compiladas, accesibilidad y matriz CI
multiplataforma).

> Registro histórico del candidato auditado. La consolidación posterior retiró
> `state_mut`, añadió recuperación entre procesos y habilitó un workspace
> combinado para documentos, datos e ingeniería. Las evidencias de este
> registro corresponden al árbol indicado abajo.

## Cambios
- `starter-documents`: `Recovery` nombra un borrador por ejecución;
  `autosave` escribe solo documentos sucios sin mover el punto de guardado;
  `recover` restaura una revisión más nueva conservando el punto de guardado;
  `discard_recovery` limpia el borrador. `History::restore` sustituye el valor
  vivo manteniendo el snapshot limpio y `Format<T>` es `Copy`.
- `application-core`: `Application::state_mut` queda para adaptadores de
  persistencia; las ediciones de dominio siguen pasando por `invoke`.
- `inspection-demo`: formato `starter-inspection`, operaciones
  `inspection.get/edit/undo/redo` y vista que usa la sesión para abrir, guardar,
  autosave y recuperar. La vista no gestiona `FileBinding`, envelopes ni
  checksums.
- `starter.plan` registra la capability `inspection-reports` y el producto
  `inspection-workbench`; el compositor ya detecta el proveedor y rechaza
  combinar workspaces.

## Checklist
- [x] `starter-documents` expone `Recovery` y la sesión con autosave y
  recuperación, con pruebas del punto de guardado.
- [x] `inspection-workbench` instala `starter-inspection` y `starter-demo` y
  gestiona el archivo solo a través de la sesión tipada.
- [x] El plan resuelve `inspection-reports` y rechaza combinar workspaces.
- [x] `verify_domain_scenarios.py --scenario inspection` pasa contra un
  workspace generado.
- [x] `unittest`, `check_docs`, `verify_templates` y los cuatro escenarios
  existentes siguen pasando.

## Evidence
- `python3 -m unittest discover -s tests`: 29 tests OK.
- `python3 tools/check_docs.py`: 34 archivos OK.
- `python3 tools/verify_templates.py`: 11 perfiles offline OK.
- `python3 tools/verify_domain_scenarios.py --evidence /tmp/domain-evidence-all.json`:
  desktop, documents, inspection, data y engineering `passed`. Generación
  154–176 ms y verificación 2.0–4.6 s por escenario con `CARGO_TARGET_DIR`
  compartido; el JSON se generó con el cambio sin commitear sobre `43401f3`.
- Cada escenario pasó `cargo fmt --all`, `cargo test` por paquete y ejecutó su
  binario por CLI o MCP. `inspection` ejerció `inspection.edit` por CLI;
  `documents` ejercitó el nuevo ciclo autosave/recuperación/descarte.
- Herramientas: `cargo-generate 0.26.0` y `tomli-w 1.2.0` vía `uv`;
  rustc `1.101.0-nightly`.
