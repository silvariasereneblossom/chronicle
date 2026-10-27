- `catchUpSubscription` with a `fromSequence` that no record on the branch
  holds, such as `currentSequence() + 1` for a subscriber that only wants
  what comes next, now replays only records from that sequence on. It used
  to start from the beginning of the log and replay the branch's whole
  history, which overflowed the subscription's buffer and dropped it.
  `Store::iter_from` likewise starts at the branch's first record at or after
  the sequence, or at the end of the log when there is none.
