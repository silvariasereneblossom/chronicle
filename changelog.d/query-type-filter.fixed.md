- `query` with a `types` filter now returns up to `limit` records of those
  types (after `offset` of them), however far apart they are in the branch.
  It used to look only at the first `limit * 4 + 100` records of the range,
  so a rare type came back short or empty: with 1,000 records and a type at
  every 200th, `query({ types: [type], limit: 5 })` returned one record. It
  now reads on in pages until enough match or the range runs out.
