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
