# Entrega reconstruida del candidato a v2.0

El parche se reconstruyó desde `ada50a578dd1535e47e61127a4a2f14f43b3191c`.
La versión previa vivía en un entorno temporal y su copia no está disponible.
Este parche tiene su propio historial; sus evidencias se regeneran con los
comandos de [validación](VALIDATION.md) y los artefactos de CI.

## Contenido

- Se conservan los once perfiles de Cargo Generate y se corrige el verificador
  para comprobar nativo y WASM como capacidades independientes.
- El compositor valida versiones, dependencias, conflictos, plataformas y
  colisiones. Produce proyectos autónomos con cuatro presets y 17 módulos.
- Los ejemplos Document Workbench, Data Explorer y Engineering Workbench
  usan un registro de operaciones tipadas desde GUI, CLI y MCP.
- El starter incluye adaptador Axum, verificación de capas, persistencia
  documental con revisión y semántica comprobada, docking, galería accesible,
  tareas, CSV, canvas, evaluación incremental y viewport wgpu.
- Los workflows validan proyectos generados. El empaquetado es manual.

Consultar [la guía del generador](GENERATOR.md), [el catálogo](MODULE_CATALOG.md),
[las fuentes](SOURCES.md) y [la migración](MIGRATION_V2.md).

## Verificación

Este repositorio no versiona las evidencias. Los comandos de
[VALIDATION.md](VALIDATION.md) las regeneran; `validate-modules.yml` produce y
sube las evidencias de composición, mientras `validate-templates.yml` valida los
once perfiles sin publicar artefactos. La ejecución de la reconstrucción
registró: los once perfiles generados y comprobados con Cargo en Linux, con WASM
para las entradas correspondientes; el workspace híbrido con pruebas nativas y
compilación WASM; capturas de galería y viewport desde wgpu offscreen —no una
sesión de ventana nativa— y un paquete CLI Linux con licencias, inventario y
checksum, generado por el workflow manual del módulo `distribution`.

El candidato aún requiere validación en modo ventana, navegador y sesión viva
del puente gráfico, además de la matriz de ejecución en Windows y macOS.
Ver también [las limitaciones](KNOWN_LIMITATIONS.md).
