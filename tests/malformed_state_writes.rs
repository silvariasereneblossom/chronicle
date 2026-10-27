//! A state update whose bytes the state can't be materialized from is
//! refused before it's recorded: recorded, it made every later read of the
//! state fail, for good. And a committed update reports success even when
//! the automatic snapshot after it fails, so a caller doesn't retry and
//! record it twice (#39).

use chronicle::{StateOperation, StateRegistration, StateStrategy, Store, StoreConfig, StoreError};
use tempfile::TempDir;

fn new_store(dir: &TempDir, strategy: StateStrategy) -> Store {
    let store = Store::create(StoreConfig {
        path: dir.path().join("store"),
        blob_cache_size: 100,
        create_if_missing: true,
    })
    .unwrap();
    store
        .register_state(StateRegistration {
            id: "items".to_string(),
            strategy,
            initial_value: None,
        })
        .unwrap();
    store
}

fn append_log() -> StateStrategy {
    StateStrategy::AppendLog {
        delta_snapshot_every: 100,
        full_snapshot_every: 10,
    }
}

/// `op` must be refused with nothing recorded, and the state must still
/// read as `[1,2]` and take a later append.
fn assert_refused(op: StateOperation) {
    let dir = TempDir::new().unwrap();
    let store = new_store(&dir, append_log());
    for item in [b"1", b"2"] {
        store
            .update_state("items", StateOperation::Append(item.to_vec()))
            .unwrap();
    }
    let head = store.current_branch().head;
    let records = store.stats().unwrap().record_count;

    let result = store.update_state("items", op);
    assert!(matches!(result, Err(StoreError::InvalidOperation(_))), "{result:?}");

    assert_eq!(store.current_branch().head, head);
    assert_eq!(store.stats().unwrap().record_count, records);
    assert_eq!(store.get_state("items").unwrap().unwrap(), b"[1,2]");
    assert_eq!(store.get_state_len("items").unwrap(), Some(2));
    store
        .update_state("items", StateOperation::Append(b"3".to_vec()))
        .unwrap();
    assert_eq!(store.get_state("items").unwrap().unwrap(), b"[1,2,3]");
}

#[test]
fn an_append_that_isnt_json_is_refused() {
    assert_refused(StateOperation::Append(b"not json".to_vec()));
    assert_refused(StateOperation::Append(Vec::new()));
}

#[test]
fn an_edit_that_isnt_json_is_refused() {
    assert_refused(StateOperation::Edit {
        index: 0,
        new_value: b"{\"text\":".to_vec(),
    });
}

#[test]
fn an_edit_escaping_a_lone_surrogate_half_is_refused() {
    // What JSON.stringify writes for a string holding a lone UTF-16 half,
    // which the JSON parser that rebuilds the state refuses.
    assert_refused(StateOperation::Edit {
        index: 1,
        new_value: br#"{"text":"a\ud800b"}"#.to_vec(),
    });
}

#[test]
fn an_append_log_set_to_anything_but_an_array_is_refused() {
    assert_refused(StateOperation::Set(b"\"x\"".to_vec()));
    assert_refused(StateOperation::Set(b"{\"a\":1}".to_vec()));
    assert_refused(StateOperation::Set(b"not json".to_vec()));
    assert_refused(StateOperation::Set(Vec::new()));
}

#[test]
fn an_append_log_set_to_an_array_and_a_snapshot_state_set_to_any_bytes_are_taken() {
    let dir = TempDir::new().unwrap();
    let store = new_store(&dir, append_log());
    store
        .update_state("items", StateOperation::Set(b"[4,5]".to_vec()))
        .unwrap();
    assert_eq!(store.get_state_len("items").unwrap(), Some(2));

    let dir = TempDir::new().unwrap();
    let store = new_store(&dir, StateStrategy::Snapshot);
    for value in [&b"\"v1\""[..], b"\x00\xffraw bytes"] {
        store
            .update_state("items", StateOperation::Set(value.to_vec()))
            .unwrap();
        assert_eq!(store.get_state("items").unwrap().unwrap(), value);
    }
}

#[test]
fn a_committed_update_succeeds_when_its_automatic_snapshot_fails() {
    // A Snapshot's bytes aren't checked at write, so this unreadable one
    // makes the full snapshot that comes due two updates later fail to
    // materialize the state.
    let dir = TempDir::new().unwrap();
    let store = new_store(
        &dir,
        StateStrategy::AppendLog {
            delta_snapshot_every: 2,
            full_snapshot_every: 1,
        },
    );
    store
        .update_state("items", StateOperation::Snapshot(b"not json".to_vec()))
        .unwrap();
    store
        .update_state("items", StateOperation::Append(b"1".to_vec()))
        .unwrap();
    assert!(!store.state_needs_snapshot("items"));

    let record = store
        .update_state("items", StateOperation::Append(b"2".to_vec()))
        .unwrap();

    assert!(store.get_record(record.id).unwrap().is_some());
    assert_eq!(store.current_branch().head, record.sequence);
    assert_eq!(store.get_state_len("items").unwrap(), Some(2));
    // The snapshot didn't happen, and is still due for the next update.
    assert!(store.state_needs_snapshot("items"));
}
