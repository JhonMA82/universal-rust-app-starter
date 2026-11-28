# Documents

Define a domain-specific format identifier, migration chain, input size
limit and validator. Keep one bounded history per open document and commit a
single semantic change when an interaction finishes. Native saves verify a
semantic round trip, checksum and expected prior content before publication.
Advisory locking coordinates cooperating writers. Keep lock sidecars beside
the destination and use paths approved by the application. Filesystems and
outside writers determine the remaining conflict and durability risks.

Run `cargo test -p starter-documents` for migration, reopen and conflict checks.

`starter-notes` is the portable note engine used by native and hybrid
workbenches. Native disk saves register the confirmed write through the
private `notes.saved` operation. Hybrid notes keep editing and history in
memory. Detached worker jobs write separate native recovery files.

For a new native document type, use `storage::Document<T>` with your
`Format<T>`. The session provides validated edits, undo/redo, reopening and
conflict-aware saves. The first save binds the file, while Save As requires
a fresh path. Successful saves mark the snapshot clean. Reopening preserves
the stored revision and starts a fresh in-memory undo history. Use the
portable `History<T>` directly when managing storage separately.

`Recovery` names a draft file in a product-owned directory. Use a stable
application key for an untitled draft and `Recovery::for_path` for a saved file.
The latter derives its key from the canonical parent and the file name. Use
`Document::bound_path` to get the path from the last successful open or save.
`Recovery::discover` lists readable drafts for the supplied format across
restarts, including their revision and original file path when available.

`Document::autosave` writes a dirty document without moving its save point.
For a bound document, the draft records the path and checksum of its saved base.
`Document::recover` restores a newer draft against that same base and keeps the
recovered edits dirty. Recovery reports a conflict if the saved base differs.
`Document::discard_recovery` removes the selected draft after an explicit
discard or a confirmed save. Capture the old recovery handle before Save As
changes the binding. Persistent recovery metadata uses the reserved
`_starter_recovery` envelope field.
