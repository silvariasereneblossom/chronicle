- `create` (`Store::create`, `JsStore.create`) now refuses a path that
  already holds a store, or any of a store's own files (`MANIFEST`,
  `records.log`, `state.bin`, `branches.bin`), with an error saying a store
  already exists there (#34). It used to build empty state registrations and
  branches over it, which its first sync or close published over the store's
  own: registered states and branches were gone, while the records stayed in
  the log. It now takes the store's lock before looking, so a `create` aimed
  at a store another process has open fails as locked, where it used to
  rewrite that store's `MANIFEST` first.
- `openOrCreate` (`Store::open_or_create`) now opens a store where its
  `MANIFEST` is and creates one otherwise, so an existing empty directory,
  such as a volume made ahead of time, gets a new store. It used to open any
  directory that existed, which failed for an empty one with a bare "No such
  file or directory". Without `create_if_missing`, such a directory reports
  `NotInitialized`.
