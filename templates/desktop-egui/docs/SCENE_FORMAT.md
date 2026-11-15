# Contrato `egui-visual-forge.scene.v1`

El manifiesto es un JSON construido desde **observaciones sobre la imagen**, no una estructura arbitraria inventada por el modelo. `init_scene.py` crea una plantilla válida, que el agente completa tras analizar la captura.

## Propiedades

| Campo | Tipo | Significado |
|---|---|---|
| `schema` | string | Debe ser `egui-visual-forge.scene.v1` |
| `reference.path` | string | Ruta a PNG, relativa al directorio del manifiesto |
| `reference.sha256` | string | SHA-256 de los bytes del PNG original; obligatorio |
| `reference.pixel_size` | `[int,int]` | Anchura y altura físicas, no puntos egui |
| `reference.pixels_per_point` | float positivo | Factor px/punto egui; requiere verificación |
| `reference.content_rect_px` | `[x,y,w,h]` | Rectángulo del contenido egui dentro del PNG |
| `reference.state` | string | Estado UI reproducible (tab, scroll, modal, etc.) |
| `reference.platform/renderer/theme/font` | string | Datos de captura o `unverified` |
| `regions[]` | array | Regiones con `id`, `rect_px`, `weight` (0.1–10), `role` |
| `regions[].quality` | object opcional | Overrides de `max_mae`, `max_changed_fraction`, `min_edge_f1` (y umbrales de detección) |
| `ignore_rects_px[]` | array | Cada elemento: `{ "rect_px": [...], "reason": "..." }` |
| `tolerances` | object | Gates y ajustes de detección globales |
| `observed_text` | array | Transcripción literal, rectángulo opcional y confianza |
| `assumed_interactions` | array | Hipótesis sobre comportamientos invisibles |
| `uncertainties` | array | Elementos no verificables de una captura |
| `matte_rgb` | `[R,G,B]` | Fondo sobre el que se compone el alfa, si hay transparencia |

## Coordenadas

Todas las cajas son enteros `[x, y, w, h]` en píxeles físicos, origen superior izquierdo, intervalo semiabierto. `reference.content_rect_px` indica el recorte lógico; el comparador **no** recorta la captura ni reescala para hacer coincidir los resultados. Los dos PNG deben contener exactamente el mismo cuadro completo. El contenido se transforma a egui con:

```text
point_x = (pixel_x - content_rect_px[0]) / pixels_per_point
point_y = (pixel_y - content_rect_px[1]) / pixels_per_point
point_w = pixel_w / pixels_per_point
point_h = pixel_h / pixels_per_point
```

Si el candidato incluye un marco de SO distinto, cambia el procedimiento de captura, no el manifiesto de la referencia después de optimizar.

## Gates de ejemplo

```json
{
  "max_mae": 12.0,
  "max_changed_fraction": 0.12,
  "min_edge_f1": 0.70,
  "pixel_delta_threshold": 18,
  "edge_threshold": 24,
  "max_mask_fraction": 0.10
}
```

- MAE escala 0–255; *menor es mejor*.
- `changed_fraction` va de 0 a 1; *menor es mejor*. Un píxel se considera cambiado si **alguno** de sus canales supera `pixel_delta_threshold` (comparación estricta `>`).
- `edge_f1` va de 0 a 1; *mayor es mejor*. Calcula bordes por gradientes RGB→luminancia con tolerancia espacial de 1 px. Es una heurística de geometría, no OCR ni percepción semántica.
- `max_mask_fraction` limita la fracción de imagen excluida; las regiones completamente enmascaradas producen un error.
- Los gates globales se aplican a toda la imagen y, por defecto, a cada región. Se permiten overrides regionales explícitos; el resultado global nunca desaparece.
- Las regiones pueden solaparse; por eso sus estadísticas **no se agregan** en una “puntuación única” ponderada. El peso solo prioriza la revisión de discrepancias.
- Los valores de ejemplo no demuestran una precisión de 95 % y no son universalmente óptimos. Fíjalos antes de comparar las alternativas.

## Más de una captura

Para diferentes pestañas, scroll, modal o modo oscuro, crea un manifiesto **por estado**, o mantén varios manifiestos versionados por pantalla. Los resultados solo se pueden comparar entre experimentos si provienen de la misma referencia y del mismo estado.

## Cuestiones no cubiertas automáticamente

El script no identifica texto, fuentes, iconos, controles ni jerarquía semántica. Esas propiedades se analizan visualmente y, si existe acceso, se verifican mediante el árbol de accesibilidad y pruebas con `egui_kittest`.
