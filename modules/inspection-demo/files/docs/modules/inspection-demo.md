# Inspection Workbench

One inspection report with a product-owned format (`starter-inspection`),
edited through the registered operations and stored by the typed document
session. `Document<T>` owns the envelope, checksums, conflict detection and
save point, and `Recovery` writes and restores dirty drafts without marking
the file saved. The view keeps no `FileBinding`, envelope or checksum logic
of its own. Autosave runs every 15 seconds; the Recovery panel restores or
discards the draft.
