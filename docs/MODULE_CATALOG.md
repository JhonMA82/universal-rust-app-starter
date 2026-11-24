# Module catalog

| Module | Targets | Dependencies |
| --- | --- | --- |
| [application-core](../modules/application-core/files/docs/modules/application-core.md) | native, wasm | — |
| [application-http](../modules/application-http/files/docs/modules/application-http.md) | native | application-core@0.1.0 |
| [application-mcp](../modules/application-mcp/files/docs/modules/application-mcp.md) | native | application-core@0.1.0 |
| [canvas](../modules/canvas/files/docs/modules/canvas.md) | native | desktop-shell@0.1.0 |
| [data](../modules/data/files/docs/modules/data.md) | native | desktop-shell@0.1.0, tasks@0.1.0 |
| [data-demo](../modules/data-demo/files/docs/modules/data-demo.md) | native | data@0.1.0, reference-runtime@0.1.0 |
| [desktop-shell](../modules/desktop-shell/files/docs/modules/desktop-shell.md) | native, wasm | — |
| [distribution](../modules/distribution/files/docs/modules/distribution.md) | native, wasm | — |
| [document-demo](../modules/document-demo/files/docs/modules/document-demo.md) | native | documents@0.1.0, reference-runtime@0.1.0 |
| [documents](../modules/documents/files/docs/modules/documents.md) | native, wasm | — |
| [engineering-demo](../modules/engineering-demo/files/docs/modules/engineering-demo.md) | native | canvas@0.1.0, viewport@0.1.0, documents@0.1.0, evaluation@0.1.0, science@0.1.0, reference-runtime@0.1.0 |
| [engineering-tools](../modules/engineering-tools/files/docs/modules/engineering-tools.md) | native, wasm | — |
| [evaluation](../modules/evaluation/files/docs/modules/evaluation.md) | native, wasm | — |
| [reference-runtime](../modules/reference-runtime/files/docs/modules/reference-runtime.md) | native | desktop-shell@0.1.0, application-mcp@0.1.0, engineering-tools@0.1.0 |
| [science](../modules/science/files/docs/modules/science.md) | native, wasm | — |
| [tasks](../modules/tasks/files/docs/modules/tasks.md) | native | — |
| [viewport](../modules/viewport/files/docs/modules/viewport.md) | native | desktop-shell@0.1.0 |

## Presets

- **minimal** — Base profile
- **application** — application-core, engineering-tools
- **workbench** — application-core, engineering-tools, documents, desktop-shell
- **engineering** — application-core, engineering-tools, documents, desktop-shell, tasks, evaluation, science, canvas, viewport
