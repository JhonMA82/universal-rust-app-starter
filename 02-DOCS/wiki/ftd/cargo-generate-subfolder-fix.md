---
author: you
branch: main
status: done
---
# Corregir el comando de cargo-generate del README

## Intent
El README recomendaba `--template-path`, que cargo-generate no acepta. El
subdirectorio de plantilla dentro del repositorio es un argumento posicional.

## Scope
In: la invocación `cargo generate --git ...` de la sección «Direct Cargo
Generate».
Not: las instrucciones de generación local (`--path`) y las dependencias de
cada perfil.

## Checklist
- [x] Reemplazar `--template-path templates/desktop-egui` por el subdirectorio posicional.
- [x] Verificar la sintaxis contra un clon git local: proyecto generado.
- [x] Verificar el comando exacto contra `https://github.com/JhonMA82/universal-rust-app-starter`: proyecto generado.
- [x] Confirmar que no quedan referencias a `--template-path` en el repositorio.

## Evidence
`cargo generate --help` (0.26.0) define `[AUTO_PATH] [SUBFOLDER]` y no incluye
`--template-path`; el subdirectorio posicional existe desde 0.8.0 según el
changelog de cargo-generate.

Comando corregido ejecutado con éxito desde `/tmp`:

```sh
cargo generate --git https://github.com/JhonMA82/universal-rust-app-starter templates/desktop-egui --name my-desktop-app
```

Salida: `✨ Done! New project created` y `Cargo.toml` presente en el proyecto
generado. Se omite el symlink `.claude/skills` porque cargo-generate no copia
symlinks; `scripts/check_agents.py` lo recrea.

## Next
Ninguno.
