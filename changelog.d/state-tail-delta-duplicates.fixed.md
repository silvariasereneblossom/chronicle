- `getStateTail` no longer returns items twice once the tail reaches into a
  delta snapshot of an append-log state. After taking items from the delta it
  went on to the Appends the delta already holds, so with the default cadence
  `getStateTail(id, 150)` on 220 items returned items 171..200 a second time
  and missed 71..100. It now skips them, as full reads do.
