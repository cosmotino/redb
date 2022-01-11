use redb::{Database, MultimapTable, ReadOnlyMultimapTable, ReadableMultimapTable};
use tempfile::NamedTempFile;

fn get_vec(table: &ReadOnlyMultimapTable<[u8], [u8]>, key: &[u8]) -> Vec<Vec<u8>> {
    let mut result = vec![];
    let mut iter = table.get(key).unwrap();
    loop {
        let item = iter.next();
        if let Some(item_value) = item {
            result.push(item_value.to_vec());
        } else {
            return result;
        }
    }
}

#[test]
fn len() {
    let tmpfile: NamedTempFile = NamedTempFile::new().unwrap();
    let db = unsafe { Database::open(tmpfile.path(), 1024 * 1024).unwrap() };
    let write_txn = db.begin_write().unwrap();
    let mut table: MultimapTable<[u8], [u8]> = write_txn.open_multimap_table(b"x").unwrap();

    table.insert(b"hello", b"world").unwrap();
    table.insert(b"hello", b"world2").unwrap();
    table.insert(b"hi", b"world").unwrap();
    write_txn.commit().unwrap();

    let read_txn = db.begin_read().unwrap();
    let table: ReadOnlyMultimapTable<[u8], [u8]> = read_txn.open_multimap_table(b"x").unwrap();
    assert_eq!(table.len().unwrap(), 3);
}

#[test]
fn is_empty() {
    let tmpfile: NamedTempFile = NamedTempFile::new().unwrap();
    let db = unsafe { Database::open(tmpfile.path(), 1024 * 1024).unwrap() };

    let write_txn = db.begin_write().unwrap();
    let mut table: MultimapTable<[u8], [u8]> = write_txn.open_multimap_table(b"x").unwrap();
    table.insert(b"hello", b"world").unwrap();
    write_txn.commit().unwrap();

    let read_txn = db.begin_read().unwrap();
    let table: ReadOnlyMultimapTable<[u8], [u8]> = read_txn.open_multimap_table(b"x").unwrap();
    assert!(!table.is_empty().unwrap());
}

#[test]
fn insert() {
    let tmpfile: NamedTempFile = NamedTempFile::new().unwrap();
    let db = unsafe { Database::open(tmpfile.path(), 1024 * 1024).unwrap() };
    let write_txn = db.begin_write().unwrap();
    let mut table: MultimapTable<[u8], [u8]> = write_txn.open_multimap_table(b"x").unwrap();
    table.insert(b"hello", b"world").unwrap();
    table.insert(b"hello", b"world2").unwrap();
    write_txn.commit().unwrap();

    let read_txn = db.begin_read().unwrap();
    let table: ReadOnlyMultimapTable<[u8], [u8]> = read_txn.open_multimap_table(b"x").unwrap();
    assert_eq!(
        vec![b"world".to_vec(), b"world2".to_vec()],
        get_vec(&table, b"hello")
    );
    assert_eq!(table.len().unwrap(), 2);
}

#[test]
fn range_query() {
    let tmpfile: NamedTempFile = NamedTempFile::new().unwrap();
    let db = unsafe { Database::open(tmpfile.path(), 1024 * 1024).unwrap() };
    let write_txn = db.begin_write().unwrap();
    let mut table: MultimapTable<[u8], [u8]> = write_txn.open_multimap_table(b"x").unwrap();
    for i in 0..5u8 {
        let value = vec![i];
        table.insert(b"0", &value).unwrap();
    }
    for i in 5..10u8 {
        let value = vec![i];
        table.insert(b"1", &value).unwrap();
    }
    for i in 10..15u8 {
        let value = vec![i];
        table.insert(b"2", &value).unwrap();
    }
    write_txn.commit().unwrap();

    let read_txn = db.begin_read().unwrap();
    let table: ReadOnlyMultimapTable<[u8], [u8]> = read_txn.open_multimap_table(b"x").unwrap();
    let start = b"0".as_ref();
    let end = b"1".as_ref();
    let mut iter = table.get_range(start..=end).unwrap();
    for i in 0..10u8 {
        let (key, value) = iter.next().unwrap();
        if i < 5 {
            assert_eq!(b"0", key);
        } else {
            assert_eq!(b"1", key);
        }
        assert_eq!(&[i], value);
    }
    assert!(iter.next().is_none());
}

#[test]
fn delete() {
    let tmpfile: NamedTempFile = NamedTempFile::new().unwrap();
    let db = unsafe { Database::open(tmpfile.path(), 1024 * 1024).unwrap() };
    let write_txn = db.begin_write().unwrap();
    let mut table: MultimapTable<[u8], [u8]> = write_txn.open_multimap_table(b"x").unwrap();
    table.insert(b"hello", b"world").unwrap();
    table.insert(b"hello", b"world2").unwrap();
    write_txn.commit().unwrap();

    let read_txn = db.begin_read().unwrap();
    let table: ReadOnlyMultimapTable<[u8], [u8]> = read_txn.open_multimap_table(b"x").unwrap();
    assert_eq!(
        vec![b"world".to_vec(), b"world2".to_vec()],
        get_vec(&table, b"hello")
    );
    assert_eq!(table.len().unwrap(), 2);

    let write_txn = db.begin_write().unwrap();
    let mut table: MultimapTable<[u8], [u8]> = write_txn.open_multimap_table(b"x").unwrap();
    table.remove(b"hello", b"world2").unwrap();
    write_txn.commit().unwrap();

    let read_txn = db.begin_read().unwrap();
    let table: ReadOnlyMultimapTable<[u8], [u8]> = read_txn.open_multimap_table(b"x").unwrap();
    assert_eq!(vec![b"world".to_vec()], get_vec(&table, b"hello"));
    assert_eq!(table.len().unwrap(), 1);

    let write_txn = db.begin_write().unwrap();
    let mut table: MultimapTable<[u8], [u8]> = write_txn.open_multimap_table(b"x").unwrap();
    table.remove_all(b"hello").unwrap();
    write_txn.commit().unwrap();

    let read_txn = db.begin_read().unwrap();
    let table: ReadOnlyMultimapTable<[u8], [u8]> = read_txn.open_multimap_table(b"x").unwrap();
    assert!(table.is_empty().unwrap());
    let empty: Vec<Vec<u8>> = vec![];
    assert_eq!(empty, get_vec(&table, b"hello"));
}