# Module catalog

| Module | Targets | Dependencies |
| --- | --- | --- |
| [application-core](../modules/application-core/files/docs/modules/application-core.md) | native, wasm | — |
| [application-http](../modules/application-http/files/docs/modules/application-http.md) | native | application-core |
| [application-mcp](../modules/application-mcp/files/docs/modules/application-mcp.md) | native | application-core |
| [canvas](../modules/canvas/files/docs/modules/canvas.md) | native | desktop-shell |
| [data](../modules/data/files/docs/modules/data.md) | native | desktop-shell, tasks |
| [data-demo](../modules/data-demo/files/docs/modules/data-demo.md) | native | data, reference-runtime |
| [desktop-shell](../modules/desktop-shell/files/docs/modules/desktop-shell.md) | native, wasm | — |
| [distribution](../modules/distribution/files/docs/modules/distribution.md) | native, wasm | — |
| [document-demo](../modules/document-demo/files/docs/modules/document-demo.md) | native | documents, reference-runtime |
| [documents](../modules/documents/files/docs/modules/documents.md) | native, wasm | — |
| [engineering-demo](../modules/engineering-demo/files/docs/modules/engineering-demo.md) | native | canvas, viewport, documents, evaluation, science, reference-runtime |
| [engineering-tools](../modules/engineering-tools/files/docs/modules/engineering-tools.md) | native, wasm | — |
| [evaluation](../modules/evaluation/files/docs/modules/evaluation.md) | native, wasm | — |
| [reference-runtime](../modules/reference-runtime/files/docs/modules/reference-runtime.md) | native | desktop-shell, application-mcp, engineering-tools |
| [science](../modules/science/files/docs/modules/science.md) | native, wasm | — |
| [tasks](../modules/tasks/files/docs/modules/tasks.md) | native | — |
| [viewport](../modules/viewport/files/docs/modules/viewport.md) | native | desktop-shell |

## Presets

- **minimal** — Base profile
- **application** — application-core, engineering-tools
- **workbench** — application-core, engineering-tools, documents, desktop-shell
- **engineering** — application-core, engineering-tools, documents, desktop-shell, tasks, evaluation, science, canvas, viewport
