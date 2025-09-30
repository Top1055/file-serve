use file_serve::db::{Db, Share}; // Adjust this path based on your actual crate structure
use std::fs::File;
use std::io::Write;

// Simple helper to create a dummy file
fn temp_file_with_size(bytes: usize) -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("demo.bin");
    let mut f = File::create(&path).unwrap();
    f.write_all(&vec![0u8; bytes]).unwrap();
    (dir, path.to_string_lossy().to_string())
}

#[test]
fn file_create_or_get_is_idempotent() {
    let db = Db::new().unwrap(); // or Db::new_in_memory() if you add that
    let (_td, p) = temp_file_with_size(1234);

    let a = db.create_or_get_file(&p).unwrap();
    let b = db.create_or_get_file(&p).unwrap();

    assert_eq!(a.id, b.id, "Same path should return same file ID");
}

#[test]
fn share_create_and_delete_works() {
    let db = Db::new().unwrap();
    let (_td, p) = temp_file_with_size(10);
    let file = db.create_or_get_file(&p).unwrap();

    let share = db
        .create_share(&Share {
            slug: "".to_string(), // slug is generated internally
            file_id: file.id.clone(),
            expires_at: None,
            max_downloads: None,
            dl_count: 0,
            password_hash: None,
            created_at: "".to_string(), // ignored
        })
        .unwrap();

    assert!(db.get_share(&share.slug).unwrap().is_some());
    assert!(db.delete_share(&share.slug).unwrap());
    assert!(db.get_share(&share.slug).unwrap().is_none());
}
