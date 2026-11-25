# Reuse decisions

This audit compares source, manifests, notices and representative consumers at
the revisions below. It records integration and maintenance tradeoffs. Upstream
test fixtures were inspected as maturity evidence. The audit also exercised
`craft-atomic` in isolation on Linux. Generated applications continue to resolve their
own dependencies and can inspect the result with `cargo tree`.

## Reproducible sources

| Project and version | Revision and files inspected |
|---|---|
| Craft Libs 0.1.0 | [05b99ee5dcd64f2c954a693ef361301214ddc3c7](https://github.com/storytold/craft-libs/tree/05b99ee5dcd64f2c954a693ef361301214ddc3c7). Workspace and package manifests, `crates/ui/README.md`, UI and platform sources and tests, `NOTICE`, `ATTRIBUTION.md`. |
| DesignCraft | [cdd650ccddcd945bc3bd050f40299955a7dd1c4b](https://github.com/storytold/designcraft/tree/cdd650ccddcd945bc3bd050f40299955a7dd1c4b). Workspace and UI manifests, `crates/ui-egui/src/app/file_dialogs.rs`, `crates/ui-egui/src/app/screenshot.rs`, and `crates/engine/src/atomic.rs`. |
| GridCraft | [33bb59307b89c3a9fc8e4f20ce7ab4818a994a04](https://github.com/storytold/gridcraft/tree/33bb59307b89c3a9fc8e4f20ce7ab4818a994a04). Workspace and UI manifests, `crates/ui-egui/src/printing.rs` and `crates/ui-egui/tests/doc_windows.rs`. |
| WordCraft | [c179a31619632d96728653f00c56246ba374e928](https://github.com/storytold/wordcraft/tree/c179a31619632d96728653f00c56246ba374e928). Workspace and UI manifests, `crates/ui-egui/src/windows.rs`, `crates/ui-egui/src/rtl.rs`. |
| ArtCraft | [9a1cc6b701e9dbec38af820507c9b33b668f54f0](https://github.com/storytold/artcraft/tree/9a1cc6b701e9dbec38af820507c9b33b668f54f0). Workspace, `artcraft-native/Cargo.toml`, `crates/desktop/artcraft/Cargo.toml`, `NOTICE`. |
| CraftKit 0.2.0 | [6aef7a012e9ae16d91443a441daabe095af5beb0](https://github.com/CipherOcto/craftkit/tree/6aef7a012e9ae16d91443a441daabe095af5beb0). Workspace, `LICENSE`, SDK and MCP/control manifests and public APIs. |
| SolidCraft 0.49.0 | [9ef680c87ecfe6ba7187955bf88212478fb44b6a](https://github.com/tangency-labs/solidcraft/tree/9ef680c87ecfe6ba7187955bf88212478fb44b6a). Workspace, `LICENSE-BRAND`, UI kit, engine, document, GPU and MCP manifests. |
| Open CAD Studio 2026.47.0 | [08be7bcff3f767bcc21738823db9c3efb977db93](https://github.com/HakanSeven12/OpenCADStudio/tree/08be7bcff3f767bcc21738823db9c3efb977db93). Workspace, `LICENSE`, `crates/ocs_save/Cargo.toml`, `docs/automation/API-SPEC.md`. |
| RustGIS 0.8.0 | [2342a2c518ad36b2e808b189b6234321cb6ea729](https://github.com/opengeos/RustGIS/tree/2342a2c518ad36b2e808b189b6234321cb6ea729). Workspace, `LICENSE`, `crates/ui-egui/Cargo.toml`, snapshot example configuration and package boundaries. |

## Craft Libs

The inspected Cargo workspace contains 204 package manifests under `crates/*`.
Its own crates use MIT OR Apache-2.0 and advertise Rust 1.95. Per-file notices
cover vendored code, test data and fonts. The graphics stack declares egui 0.37
and wgpu 30, with workspace patches for the egui family and winit. DesignCraft
and GridCraft pin the same egui fork revision `6d518fc799681571963842f95b55a6b68431b15a`.
WordCraft uses a different fork pin `f5025284445ea19c35f746d96fc0b75468b3f560`.
`craft-ui` documents crates.io support with reduced behavior for portions of RTL,
font baselines and IME. Its suite's full behavior depends on those patches.

| Crates and capability | Dependencies, platform and usage evidence | Decision and maintenance |
|---|---|---|
| `craft-ui` for themed controls, icon buttons, tabs, docking, printing and capture | egui/epaint 0.37, craft-fonts-build, craft-i18n and skrifa. Optional native test, font and window features. DesignCraft uses the combo box, layout and capture slot. GridCraft uses combo, printing and ribbon controls. The crate has docking workflow, render and web font tests. | Retain egui primitives, the starter design tokens and egui_dock. The starter already owns dock layout, active operations and restoration. A full switch would add font/i18n policy and a second component or docking model. Adopt a small component only when a project needs behavior unavailable in official egui. |
| `craft-window`, `craft-quit`, `craft-menu`, `craft-keymap` for shell lifecycle and commands | egui optional in window/quit, native Wayland and AppKit branches, optional storage in window. Menu adds quit/keymap and a macOS backend through muda and AppKit. DesignCraft uses title-bar integration, GridCraft exercises document window placement, and WordCraft uses the window list and shortcut overrides. | Keep `starter-desktop` for workspace lifecycle and the shared operation registry for commands. Apps can add native menu or app-wide quit adapters where their lifecycle requires them. Bringing in all four would duplicate routing and require replacing close, panel and persistence policies. |
| `craft-dialogs`, `craft-platform` for nonblocking file prompts, Linux/BSD appearance and browser file downloads | Dialogs uses raw-window-handle 0.6.2 and rfd 0.17.2 with XDG portal, plus AppKit on macOS. DesignCraft's handlers map queued dialog answers to app requests and tests inject responses. Platform uses optional D-Bus or browser bindings and has dialog lifecycle and browser download tests. | Preserve in-app confirmations and the existing native/web boundaries. `craft-dialogs` is a candidate for a selectable OS file-picker adapter, with per-platform checks for portal availability, window ownership and cancellation. Avoid turning these requirements into dependencies of every desktop app. |
| `craft-i18n` and `craft-settings` for locale and preferences | i18n adds ICU providers, Unicode folding, bidirectional text and platform locale APIs. Settings uses serde/TOML and craft-atomic, with native directories and recovery tests. WordCraft drives both via its suite facade and embeds suite catalogs. | Keep product-owned labels and the existing shell preference store. The suite catalogs are tailored to its apps. Locale-aware formats and full translated catalogs remain a distinct need for multilingual products. |
| `craft-atomic`, `craft-document-bundle`, `craft-doc` for low-level persistence and file formats | Atomic uses standard filesystem APIs with Apple rustix and has permission/write tests. DesignCraft wraps it for native writes. Document bundle adds ZIP, and craft-doc decodes three suite-specific formats. | Preserve the existing document revision, migration, save verification and conflict contract. Atomic replacement alone would not substitute for that contract. Re-evaluate `craft-atomic` beneath the persistence adapter when its durability or retry guarantees are needed. Suite bundle and format crates add unnecessary format obligations to the generic template. |
| `craft-control`, `craft-control-protocol`, `craft-jsonrpc`, `craft-mcp`, `craft-mcp-server` | Bounded JSON and native loopback/stdio pieces. The optional agents integration adds its own suite dependencies. | Preserve `starter-core` as the source of public operation metadata and the existing rmcp/Axum transports. A parallel control channel or MCP implementation would duplicate schemas, session checks and security review. |
| `craft-testkit`, `craft-xtask`, `craft-update` | Source guards and deadlines, step runner and reports, or platform-specific installer and self-update flows. Testkit has guard fixtures. Updates add network, packaging and platform dependencies. | Retain Cargo checks, existing evidence schema, `xtask` and Visual Forge. Installer and update ownership remain with each generated product. |
| `craft-gpu`, `craft-gpu-wgpu`, `craft-frame` | Provider-neutral GPU jobs, wgpu 30 with native backends and browser WebGPU, or video frame/color abstractions. Frame stores video compositor data and does not own application shell frames. | Keep the existing viewport adapter on wgpu. Generic GPU job and video frame abstractions add maintenance for capabilities outside the starter's viewport contract. |

The standalone `craft-atomic` Linux check compiled `src/lib.rs` as an rlib
with `rustc 1.100.0 --edition=2024`, then used `rustc --test` on that source
and on `tests/atomic_tests.rs` and `tests/permissions.rs` with
`--extern craft_atomic=<rlib>` for the integration suites. It ran 35 passing
tests with two POSIX ACL cases ignored by the upstream harness. This exercise
supports the low-level candidate status. It leaves the starter's document
format and its compatibility tests unchanged.

## Other upstreams

| Project, packages and license | Compatibility and decision |
|---|---|
| CraftKit `craftkit-sdk`, `craftkit-control`, `craftkit-mcp` (MIT OR Apache-2.0) | Rust 1.93 and egui 0.33 in the workspace. Its SDK owns sessions, command schemas and the plug-in ecosystem. The original starter registry follows the shared operation-catalog pattern. Keep the starter's registry, which already powers CLI, UI and opt-in MCP. A second SDK would recreate session and extension contracts. |
| ArtCraft `artcraft_native` and desktop app (MIT OR Apache-2.0 for own code) | The native manifest uses eframe 0.36/Glow. The other desktop app uses Tauri 2.11.2. Inspect composition and cross-platform packaging patterns. Do not transplant the runtime, bundled media or decorative assets. Its notice identifies third-party media and marks outside the code license. |
| SolidCraft `solidcraft-ui-kit`, `solidcraft-look`, `solidcraft-theme`, `solidcraft-doc`, `solidcraft-engine`, `solidcraft-gpu` and `solidcraft-mcp` | Rust 1.95, egui 0.37 with local workspace patches, and wgpu 30. Use neutral viewport/application boundaries as inspiration. Keep the existing generic viewport, domain and document contracts. The geometry code's dual license does not extend to its UI kit, look, tokens or marks. `LICENSE-BRAND` expressly requires written permission to reuse those portions externally. |
| Open CAD Studio `ocs_save`, `ocs_plugin_api`, `ocs_web_worker` (repository GPL-3.0) | The app combines pinned Iced branches and a domain-specific DWG/DXF document codec. Its documented revision checks and all-or-nothing operations inform the starter's contract. No source is imported into the permissive template. |
| RustGIS `rustgis-ui-egui`, `rustgis-engine`, `rustgis-assistant` (Apache-2.0) | Rust 1.92, egui/eframe/egui-wgpu/egui_kittest 0.37, native rfd and a WebAssembly route. The UI directly depends on the GIS engine and assistant and uses tiles in its test graph. Its test manifest demonstrates snapshot coverage. Keep the pattern of a separable engine and in-app viewport integration. Preserve starter adapters independently of GIS state. |

## Direct dependencies kept

The starter integrates egui/eframe 0.37 and egui_dock 0.22 for shared UI,
egui_extras 0.37 and egui_plot 0.38 for tables and charts, and wgpu 30 through
the optional engineering viewport. `egui_kittest` and Visual Forge cover
development verification. The data adapter joins reusable UI with a domain
dataset. `petgraph` and `uom` cover graph algorithms and dimensions in the
engineering preset. `rmcp` and Axum remain protocol and HTTP adapters around
the same operation registry. Version constraints and platform features live in
the generated Cargo manifests. See [sources and attribution](SOURCES.md).

The generated application owns any new fonts, icon assets and publication
notices. No Craft, SolidCraft, Open CAD Studio or RustGIS source or assets are
bundled by these decisions. Prefer a narrow released dependency where it fits.
For a Git dependency, pin its full revision and record the license, platforms and
tested compatibility before adopting it.
