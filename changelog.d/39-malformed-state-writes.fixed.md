- A state update whose bytes its state can't be rebuilt from is now refused,
  with nothing recorded: an `appendToState` or `editStateItem` of bytes that
  don't parse as JSON (including JSON escaping a lone UTF-16 surrogate half,
  #52), and a `setState` or `setStateJson` of anything but a JSON array on an
  `append_log` state, an empty value included. Each used to be recorded, and
  from then on every read of the state's value threw, across reopens, whatever
  was written after it. A state that already holds such a record stays
  unreadable.
- A state update that was recorded no longer throws when the automatic
  snapshot after it fails (#39). The caller couldn't tell the update had been
  committed, and a retry recorded it twice. The failure is now logged, and the
  next update of that state tries the snapshot again.
