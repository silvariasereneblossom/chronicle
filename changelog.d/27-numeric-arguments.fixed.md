- Numeric arguments from JavaScript are now range-checked, and one out of
  range throws an `Error` naming it, such as
  `Invalid limit: -1 (a non-negative integer)` (#27). They used to be cast
  unchecked: `query({ limit: -1 })`, `query({ offset: -1 })`,
  `query({ limit: 1e15 })` and a negative `blobCacheSize` aborted the Node
  process, a `bufferSize` of `1e9` could too, and elsewhere a negative number
  wrapped to a huge one and the call answered something else.
  - Counts, indexes, offsets, sizes, sequences and timeouts must be
    non-negative integers: `query`'s `limit`, `offset`, `fromSequence` and
    `toSequence`, `queryStateIndexRange` and `queryStateIndexEq`'s `limit` and
    `offset` (which used to wrap modulo 2^32), `getStateSlice`,
    `getStateTail`, `editStateItem`, `redactStateItems`, `getStateAt`,
    `getStateJsonAt`, `createBranchAt`, `treeDiff`,
    `pollSubscriptionTimeout`, `subscribe`'s `maxSnapshotBytes` and
    `fromSequence`, the store config's `blobCacheSize`, and a tree entry's
    `size` and `mode`. A whole number larger than the store holds still
    means all of it: `query({ limit: 1e15 })` returns every record.
  - `deltaSnapshotEvery` and `fullSnapshotEvery` must be integers from 1 to
    4294967295, and `subscribe`'s `bufferSize` an integer from 1 to 1048576.
  - `getStateItemJson` still returns `null` for a negative index.
  - `query` with a `fromSequence` past its `toSequence` returns no records,
    where it panicked.
- From Rust, `Store::query_range` no longer reserves memory for the whole
  `limit`, and returns nothing for a `from` past its `to` rather than
  panicking; a store's blob cache no longer reserves room for
  `blob_cache_size` entries up front; and a subscription's buffer is at most
  `SubscriptionConfig::MAX_BUFFER_SIZE` (1,048,576) events, where a larger
  `buffer_size` panicked or asked for more memory than exists.
