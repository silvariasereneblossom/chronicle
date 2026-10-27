- Opening a store whose `state.bin` or `branches.bin` is cut short or
  damaged now fails with a `Corruption` error that names the file. A length
  field naming more bytes than the file holds used to be allocated as given
  before reading, which for a large enough value panicked or asked for more
  memory than exists and aborted the process, and otherwise failed with a
  bare "failed to fill whole buffer". A `branches.bin` naming a current
  branch it doesn't hold is refused the same way, where the store used to
  open and then panic on its first use of the current branch.
