# Inspection Workbench

One inspection report with a product-owned format (`starter-inspection`),
edited through the registered operations and stored by the typed document
session. `Document<T>` owns the envelope, checksums, conflict detection and
save point, and `Recovery` writes and restores dirty drafts without marking
the file saved. The view keeps no `FileBinding`, envelope or checksum logic
of its own. Autosave runs every 15 seconds; the Recovery panel restores or
discards the draft.

The Recovery panel lists previous-process drafts from `.inspection-recovery`
in the working directory. Saved reports derive their key from the original
file location. Each untitled session gets its own draft key, and recovery
continues that draft. Run subsequent sessions from the same directory to
discover it. Saving preserves drafts until you explicitly discard them.
A draft older than its saved base cannot override that saved report.

Discard checks that the selected draft has not changed since discovery. If
another session autosaves first, refresh the list before deciding to discard it.
