- A branch made by `createBranchAt` after a Redact, Edit or Set of an
  append-log state no longer brings back what that operation removed or
  replaced once the branch's first delta snapshot comes due (by default after
  100 more operations). The new branch's chain head started with no record of
  the operation, so it took a delta snapshot, which gathers every Append since
  the last snapshot, the redacted and replaced ones among them, and reads then
  skipped the operation behind it: redacted items came back, edits were
  undone, and `getStateLen` disagreed with the value. The branch now takes
  that record from the parent's history up to the branch point. A delta
  snapshot that such a branch wrote before this fix stays in its chain, and
  the branch reads as it did.
- `createBranchAt` now gives the new branch every state the parent has
  written, as `createBranch` does, including states written without a
  registration. It used to copy registered states only, so such a state read
  as absent on the new branch.
