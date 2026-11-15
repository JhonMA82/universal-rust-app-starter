# eframe starter 0.37

Una aplicación **mínima y deliberadamente genérica** con estado controlado, barra lateral, viewport central y controles egui auténticos. Sirve como referencia de código y prueba de interacción; no representa una réplica de ninguna captura.

```bash
cargo fmt --check
cargo check
cargo test
cargo run
```

No se incluye un PNG objetivo; para entrenar una reconstrucción, proporciona una referencia externa y crea su `scene.json`. La prueba `egui_kittest` verifica que un checkbox cambia de estado, pero no establece fidelidad visual.

**Verificación:** compilado y probado con Rust 1.100.0 y eframe/egui 0.37.0 (`cargo check --all-targets`, `cargo test`) al integrar el toolkit. La prueba `egui_kittest` verifica interacción, no fidelidad visual.
