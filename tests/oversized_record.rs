//! A record whose type, payload or link lists are longer than the log's
//! frame can describe is refused before anything is written. Its lengths and
//! counts used to be cut to the frame's field widths while the full bytes were
//! written, so the record was framed wrongly and the next open refused the
//! log as corrupt.

use chronicle::{RecordId, RecordInput, Store, StoreConfig, StoreError};
use tempfile::TempDir;

fn config(dir: &TempDir, create: bool) -> StoreConfig {
    StoreConfig {
        path: dir.path().join("store"),
        blob_cache_size: 100,
        create_if_missing: create,
    }
}

/// Appends `oversized`, which must be refused naming `what`, then a record
/// that fits, and reopens the store: it must open, and hold only the record
/// that fits, under the next ID.
fn assert_refused_and_store_opens(oversized: RecordInput, what: &str) {
    let dir = TempDir::new().unwrap();
    let store = Store::create(config(&dir, true)).unwrap();
    let first = store.append(RecordInput::raw("note", b"before".to_vec())).unwrap();

    match store.append(oversized) {
        Err(StoreError::InvalidOperation(message)) => {
            assert!(message.contains(what), "{message}")
        }
        Err(other) => panic!("expected a refusal naming {what}, got {other:?}"),
        Ok(record) => panic!("expected a refusal naming {what}, got record {}", record.id.0),
    }
    let after = store.append(RecordInput::raw("note", b"after".to_vec())).unwrap();
    assert_eq!(after.id.0, first.id.0 + 1, "the refused record took no ID");
    store.sync().unwrap();
    drop(store);

    let store = Store::open(config(&dir, false)).unwrap();
    assert!(store.recovery().is_none());
    assert_eq!(store.stats().unwrap().record_count, 2);
    assert_eq!(store.get_record(after.id).unwrap().unwrap().payload, b"after");
}

#[test]
fn a_record_type_longer_than_65535_bytes_is_refused() {
    assert_refused_and_store_opens(
        RecordInput::raw("t".repeat(65_536), b"payload".to_vec()),
        "type",
    );
}

#[test]
fn a_record_with_more_than_65535_causes_is_refused() {
    let ids = (1..=65_536).map(RecordId).collect();
    assert_refused_and_store_opens(
        RecordInput::raw("note", b"payload".to_vec()).with_caused_by(ids),
        "caused_by",
    );
}

#[test]
fn a_record_with_more_than_65535_links_is_refused() {
    let ids = (1..=65_536).map(RecordId).collect();
    assert_refused_and_store_opens(
        RecordInput::raw("note", b"payload".to_vec()).with_linked_to(ids),
        "linked_to",
    );
}

#[test]
fn a_record_at_every_limit_is_written_and_read_back() {
    let dir = TempDir::new().unwrap();
    let store = Store::create(config(&dir, true)).unwrap();
    let ids: Vec<RecordId> = (1..=65_535).map(RecordId).collect();
    let record = store
        .append(
            RecordInput::raw("t".repeat(65_535), b"payload".to_vec())
                .with_caused_by(ids.clone())
                .with_linked_to(ids.clone()),
        )
        .unwrap();
    store.sync().unwrap();
    drop(store);

    let store = Store::open(config(&dir, false)).unwrap();
    let read = store.get_record(record.id).unwrap().unwrap();
    assert_eq!(read.record_type.len(), 65_535);
    assert_eq!(read.caused_by, ids);
    assert_eq!(read.linked_to, ids);
}
