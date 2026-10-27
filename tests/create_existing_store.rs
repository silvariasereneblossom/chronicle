//! `Store::create` refuses a directory that already holds a store, which it
//! used to overwrite with empty state registrations and branches (#34), and
//! `open_or_create` tells a store from a directory by its MANIFEST, so a
//! directory made ahead of time gets a new store.

use chronicle::{StateOperation, StateRegistration, StateStrategy, Store, StoreConfig, StoreError};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn config(path: &Path, create: bool) -> StoreConfig {
    StoreConfig {
        path: path.to_path_buf(),
        blob_cache_size: 100,
        create_if_missing: create,
    }
}

/// A store with a registered state of one item and a second branch, closed.
fn closed_store(path: &Path) {
    let store = Store::create(config(path, true)).unwrap();
    store
        .register_state(StateRegistration {
            id: "items".to_string(),
            strategy: StateStrategy::AppendLog {
                delta_snapshot_every: 100,
                full_snapshot_every: 10,
            },
            initial_value: None,
        })
        .unwrap();
    store
        .update_state("items", StateOperation::Append(b"1".to_vec()))
        .unwrap();
    store.create_branch("side", None).unwrap();
    store.sync().unwrap();
}

fn assert_store_intact(path: &Path) {
    let store = Store::open(config(path, false)).unwrap();
    assert_eq!(store.get_state_len("items").unwrap(), Some(1));
    assert_eq!(store.list_branches().len(), 2);
}

fn refusal(result: chronicle::Result<Store>) -> String {
    match result {
        Err(StoreError::InvalidOperation(message)) => message,
        Err(other) => panic!("expected a refusal, got {other:?}"),
        Ok(_) => panic!("expected a refusal, got a store"),
    }
}

#[test]
fn create_refuses_a_directory_that_holds_a_store() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store");
    closed_store(&path);
    let state = fs::read(path.join("state.bin")).unwrap();
    let branches = fs::read(path.join("branches.bin")).unwrap();

    let message = refusal(Store::create(config(&path, true)));
    assert!(message.contains("already exists"), "{message}");
    assert!(message.contains("MANIFEST"), "{message}");

    assert_eq!(fs::read(path.join("state.bin")).unwrap(), state);
    assert_eq!(fs::read(path.join("branches.bin")).unwrap(), branches);
    assert_store_intact(&path);
}

#[test]
fn create_refuses_a_store_that_has_lost_its_manifest() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store");
    closed_store(&path);
    fs::remove_file(path.join("MANIFEST")).unwrap();
    let state = fs::read(path.join("state.bin")).unwrap();

    refusal(Store::create(config(&path, true)));
    refusal(Store::open_or_create(config(&path, true)));

    assert_eq!(fs::read(path.join("state.bin")).unwrap(), state);
    assert!(!path.join("MANIFEST").exists());
}

#[test]
fn create_of_an_open_store_is_refused_as_locked() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store");
    closed_store(&path);
    let store = Store::open(config(&path, false)).unwrap();

    assert!(matches!(
        Store::create(config(&path, true)),
        Err(StoreError::Locked)
    ));
    drop(store);
    assert_store_intact(&path);
}

#[test]
fn open_or_create_creates_a_store_in_an_empty_directory() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("volume");
    fs::create_dir(&path).unwrap();

    let store = Store::open_or_create(config(&path, true)).unwrap();
    store
        .update_state("items", StateOperation::Append(b"1".to_vec()))
        .unwrap();
    drop(store);

    let store = Store::open_or_create(config(&path, true)).unwrap();
    assert_eq!(store.get_state_len("items").unwrap(), Some(1));
}

#[test]
fn open_or_create_without_create_if_missing_reports_an_empty_directory_uninitialized() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("volume");
    fs::create_dir(&path).unwrap();

    assert!(matches!(
        Store::open_or_create(config(&path, false)),
        Err(StoreError::NotInitialized)
    ));
}
