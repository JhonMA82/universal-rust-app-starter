# Flujo operativo de reconstrucción

## Fase 0: preparación

No utilices assets ni componentes de proyectos previos a menos que la captura de referencia o el usuario lo justifique. Registra proyecto, versión de egui, sistema operativo, escala, fuente, renderer, tamaño y captura de prueba.

## Fase 1: inspección de la captura

1. Genera `scene.json` y conserva la referencia inalterada.
2. Comprueba si la imagen contiene barra de título del SO, scrollbar, sombras externas o margen de ventana.
3. Identifica regiones padre e hijos, texto visible exacto y ubicación de widgets.
4. Asigna estado visible y diferencias desconocidas. Documenta capturas adicionales necesarias para determinar hover/clic/scroll.
5. Distingue lo medido (evidencia) de lo inferido (hipótesis).

## Fase 2: implementación base

1. App Rust/eframe con versión fija y estado estable.
2. Grandes paneles y viewport: comparar primer render con la captura antes de introducir detalles.
3. Tokens visuales medidos: fondo, superficies, texto, bordes, radios y espacios.
4. Widgets reales; no simular controles con pintura de formas no interactivas.
5. Texto e iconos; reportar assets desconocidos.

## Fase 3: captura controlada

Mismo tamaño físico, mismo contenido de ventana y misma escala efectiva. Evita hover y animaciones no presentes en la referencia. Captura de la aplicación real. Comprueba que el archivo no se redimensionó, reencuadró ni alteró.

## Fase 4: comparación

`visual_check.py` rechaza dimensiones distintas, referencias modificadas, cajas inválidas y máscaras abusivas. Genera:

- `report.json`: métricas y gates de todas las regiones.
- `report.md`: tabla human-readable.
- `overlay.png`: mezcla 50:50 de referencia/candidato, diagnóstica.
- `heatmap.png`: intensidad de diferencias, máscara en gris.
- `comparison.png`: imágenes originales lado a lado y diferencias.

El comparador **no** optimiza ni cambia el código Rust. La skill `egui-visual-optimization` se encarga de la hipótesis causal y el siguiente cambio.

## Fase 5: corrección por hipótesis

Ejemplo: “La línea vertical del panel está 16 px a la derecha; los demás elementos del panel están desplazados; probable exceso de ancho del `SidePanel`; cambiar ancho lógico antes de tocar padding de controles”.

- Registra métricas anteriores.
- Modifica únicamente el conjunto de propiedades implicado.
- Recompila, ejecuta y captura.
- Comprueba que no retroceden regiones previamente aceptadas.
- Mantén los gates sin cambios.

## Fase 6: aceptación

La aceptación exige además pruebas de comportamiento (interacciones, accesibilidad, resize). Reportar `cargo check` como equivalente a “replicación visual comprobada” sería incorrecto.

## Experimentos A/B de skills

Mismo conjunto de casos, mismo modelo y ambiente, presupuesto similar y número de rondas medido. Una condición no puede tener capturas más sencillas. Los casos deben coincidir por SHA-256, estado y tamaño. Usa `visual_check.py --tag ... --case-id ...` y `evaluate_runs.py`.
