---
name: inspection-documents
description: Extend the typed Inspection Workbench report, registry operations, desktop editor, migrations, and recovery. Use when changing report fields, check rules, or the document lifecycle.
---

# Extend inspection documents

1. Read the [typed domain](../../../crates/starter-inspection/src/lib.rs) and [desktop view](../../../crates/starter-demo/src/lib.rs). Put fields and validation in the domain. Keep uncommitted input in the view.
2. Change the report using `inspection.edit` with the document revision observed by the editor. Test invalid data and stale edits. Discover the updated schema through the installed automation transport when selected.
3. Keep filesystem actions in the application adapter. Reuse `storage::Document` for history, expected checksums, save and reopen. Store recovery drafts using the separate `storage::Recovery` location.
4. Run `cargo test -p starter-inspection -p starter-demo -p starter-documents`. Change serialized fields with a compatible default or a versioned format migration and cover older files with a test.

For the lifecycle contract, read [document storage](../../../docs/modules/documents.md). For the rest of the composition, open the [infrastructure inventory](../../../infrastructure.json).
