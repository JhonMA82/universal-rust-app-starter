# Modular generation

See the [product guide](PRODUCT_GUIDE.md) for capability-first creation.
The previous compositor interface remains available for explicit module selection.

Install Cargo Generate 0.26.0 and Python 3.11 or later. Install the composer's
pinned TOML writer with `python3 -m pip install -r generator/requirements.txt`.

```sh
python3 generator/compose.py --profile cli --preset application --name my-cli --destination ../my-cli
python3 generator/compose.py --reference document-workbench --name notes --destination ../notes
python3 generator/compose.py --reference data-explorer --name data-lab --destination ../data-lab
python3 generator/compose.py --reference engineering-workbench --name box-lab --destination ../box-lab
```

The profile defines the base application and its entrypoints. Presets select
modules. Repeat `--module id` or `--module id@version` to add capabilities.
`--target native` and `--target wasm` record build intent independently.
Every module must support all entrypoints shipped by the selected profile.
`--feature package/feature` records an available Cargo feature. Pass the
feature separately to Cargo when building.

Modules declare versions, dependencies, target and profile compatibility,
conflicts, files, path dependencies, license, features, documentation and
checks. The resolver reports invalid requests before publication. Manifests
are merged structurally, file paths are checked and every composition runs
Cargo metadata and the agent readiness checker before moving into place.
Adapters check each source anchor for a single match. A changed or ambiguous
anchor aborts generation and leaves the destination unpublished. The composer
checks declared features against Cargo metadata after the manifest merge.
The composition guide is derived from the installed entrypoints and modules.

Direct Cargo Generate remains available for every original profile.
A generated project owns its source and does not require the starter to build.
Resolve and retain a Cargo.lock when taking a project into production.

## Effective entrypoints

Minimal keeps the original profile programs. Application provides a portable
settings engine connected to CLI, web controls, service endpoints or an
exported library API as appropriate. The MCP client keeps its client as the
default executable and adds a separate operations companion.

Native Workbench and Engineering use the same document and engineering
providers selected by the corresponding references. Selecting the data module
on a native egui profile selects the Data Explorer provider. A request for at
least two of the document, data and engineering views uses the combined tank
workspace. The composer rejects incompatible explicit legacy entrypoints.
Hybrid Workbench uses the same typed notes and history
in a browser-compatible view. That view retains content in memory and provides
no native file picker or native filesystem writes.

A profile adapter installs executable entrypoints. Library and development
modules make their API or tooling available without implicitly starting a
listener or starting a job. The generated report lists entrypoints separately
from installed modules. Targets and optional Cargo features record intent.
The compatibility check always covers the profile's retained entrypoints.
The composer requires rustfmt for integrated builds.
