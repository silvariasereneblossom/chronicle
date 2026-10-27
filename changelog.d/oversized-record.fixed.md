- Appending a record whose type is longer than 65,535 bytes, whose payload is
  longer than 4,294,967,295 bytes, or that names more than 65,535
  `causedBy` or `linkedTo` IDs now fails with an error that names the field,
  and writes nothing. Such a record used to be written with its length or
  count cut to the log's 16- or 32-bit field while all its bytes followed,
  so the append succeeded, and the next open refused the store as corrupt
  once any record came after it. The payload limit covers state updates too,
  such as a `setState` of more than 4 GiB. A store holding such a record
  from before this fix still fails to open.
