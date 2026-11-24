# Distribution

Generate and commit the project lockfile before running
`python scripts/package_release.py`. A manual workflow packages native builds
and uploads only workflow artifacts. For browser packaging, build using the
profile's Trunk or Dioxus command, then pass `--web-dist <directory>`.
Bundles contain SHA-256 sidecars, Cargo dependency inventory, project licenses
and default egui font notices when applicable. Installer signing and live
deployment belong to the generated application.
