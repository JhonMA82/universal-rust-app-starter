# Native tasks

Keep I/O and CPU pools separate where both compete. Workers report progress in
basis points and check cancellation. Poll completion with the current revision.
Run `cargo test -p starter-tasks` for concurrency and stale results.

Use `TaskSlot<T, E>` when a panel owns one active job. Submit an immutable
snapshot with its application revision, render `progress()`, and poll with
the current revision. The slot handles cleanup after completion. A pending
job is cancelled when its owning view is dropped.
