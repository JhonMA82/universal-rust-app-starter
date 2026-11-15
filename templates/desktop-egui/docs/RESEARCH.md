# Fuentes consultadas y decisiones de diseño

Fecha de revisión: 11 de noviembre de 2026.

## Fuentes primarias

1. **Agent Skills Specification** — https://agentskills.io/specification  
   Se utiliza el formato `SKILL.md` con frontmatter `name`, `description`, instrucciones cortas y referencias externas opcionales. El nombre coincide con el directorio, sin mayúsculas.
2. **OpenCode Skills** — https://opencode.ai/docs/skills  
   Soporte explícito para `.agents/skills/` y `.opencode/skills/`; por ello el paquete usa un único origen de skills portables en `.agents/skills/`.
3. **egui releases** — https://github.com/emilk/egui/releases  
   Versión estable 0.37.0 publicada el 15 de octubre de 2026. Se fija esta versión como baseline para evitar mezclar firmas antiguas.
4. **eframe 0.37 `run_native`** — https://docs.rs/eframe/0.37.0/eframe/fn.run_native.html  
   Firma `run_native(..., Box::new(|cc| Ok(Box::new(...))))`; `eframe::App::ui` toma `&mut egui::Ui` en la versión fijada.
5. **egui_kittest README** — https://github.com/emilk/egui/blob/main/crates/egui_kittest/README.md  
   AccessKit, pruebas de interacción, `Harness::new_ui`, snapshots con las features `snapshot` y `wgpu`. Los snapshots son de regresión, no automáticamente capturas de referencia del usuario.
6. **egui_inspection README** — https://github.com/emilk/egui/blob/main/crates/egui_inspection/README.md  
   Protocolo de inspección para leer árbol, eventos, screenshot y resize. El screenshot puede necesitar ventana visible; el servicio de inspección debe protegerse en loopback.
7. **egui_mcp docs** — https://docs.rs/egui_mcp/latest/egui_mcp/  
   Puente MCP opcional para una app egui en ejecución. El paquete no presupone que el MCP ya está configurado.

## Decisiones de ingeniería propias

- **No depender de una skill web:** el pipeline trata como objeto primario la captura física y el código egui, no un DOM ni CSS.
- **No mezclar evidencia con inferencia:** rectángulos, texto y estados visibles provienen de la captura; comportamiento no observado se etiqueta como supuesto.
- **No utilizar una “similitud global mágica”:** MAE, fraction changed y F1 de bordes se acompañan de descomposición regional, diff visual y pruebas funcionales.
- **La referencia no se altera:** checksum obligatorio y validación estricta de dimensiones. La traducción espacial se ofrece solamente como diagnóstico, no para mejorar el score oficial.
- **Skills con responsabilidades específicas:** carga progresiva del conocimiento en lugar de un único prompt gigantesco.
- **Eval A/B por caso emparejado:** la prueba de una mejora requiere comparación de condiciones con mismas referencias, estado y entorno.

## Limitaciones conocidas

- El comparador es útil para localizar diferencias, pero no evalúa por sí solo utilidad, accesibilidad, semántica ni calidad del código.
- Se utiliza F1 de bordes con tolerancia de 1 píxel, no una métrica perceptual aprendida (LPIPS/CLIP). Esto evita descargas pesadas y favorece auditabilidad, a costa de sensibilidad a antialiasing.
- El script asume PNG con composición alfa explícita. No hace conversión de perfil ICC ni corrección avanzada de gamma/color. Si eso importa, normaliza la canalización de captura **antes** de medir.
- El código Rust de ejemplo es ilustrativo y requiere `cargo` disponible; los tests Python no afirman que el proyecto Rust compiló.
