# Modular generation

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

Direct Cargo Generate remains available for every original profile.
A generated project owns its source and does not require the starter to build.
Resolve and retain a Cargo.lock when taking a project into production.
