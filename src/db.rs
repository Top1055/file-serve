// src/db.rs
use argon2::password_hash::{Error as PwHashError, PasswordHash, PasswordVerifier, SaltString};
use argon2::{Argon2, PasswordHasher};
use rand_core::OsRng;
use rusqlite::{params, Connection, OptionalExtension};

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub id: String,
    pub abs_path: String,
    pub name: String,
    pub size_bytes: i64,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct Share {
    pub slug: String,
    pub file_id: String,
    pub expires_at: Option<String>,
    pub max_downloads: Option<i64>,
    pub dl_count: i64,
    pub password_hash: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct PublicShare {
    pub slug: String,
    pub file_name: String,
    pub file_size: i64,
    pub created_at: String,
    pub dl_count: i64,
    pub max_downloads: Option<i64>,
    pub expires_at: Option<String>,
    pub password_required: bool,
}

const SLUG_SIZE: usize = 8;
fn gen_slug(len: usize) -> String {
    use rand::{distr::Alphanumeric, Rng};
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

// ————— Password Hashing —————

/// # Errors
///
/// broken salt cannot fail here
/// if the input is gigabytes of data, it can overflow
/// if the OS's rng fails, this can trip up
/// using Argon2 default, so no config errors
pub fn hash_password(password: &str) -> Result<String, PwHashError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash);
    if let Ok(parsed) = parsed_hash {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    } else {
        false
    }
}

#[derive(Debug)]
pub struct Db {
    con: Connection,
}

impl Db {
    /// # Errors
    ///
    /// Failing to write to the file
    pub fn new() -> Result<Self, rusqlite::Error> {
        let con = Connection::open("data.db")?;
        // Tells the DB to enforce FK rules
        con.pragma_update(None, "foreign_keys", true)?;

        // Tables
        con.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS file (
                id          TEXT PRIMARY KEY,
                abs_path    TEXT NOT NULL UNIQUE,
                name        TEXT NOT NULL,
                size_bytes  INTEGER NOT NULL,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS share (
                slug            TEXT PRIMARY KEY,
                file_id         TEXT NOT NULL REFERENCES file(id) ON DELETE CASCADE,
                expires_at      TEXT,
                max_downloads   INTEGER,
                dl_count        INTEGER NOT NULL DEFAULT 0,
                password_hash   TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;

        Ok(Self { con })
    }

    // ————— file CRUD —————

    /// # Errors
    ///
    /// Will error if failing to resolve path,
    /// lacking permissions to access file's metadata,
    /// unable to write to db
    /// Will also fail if the file size is so large it exceeds u32 or i64
    pub fn create_or_get_file(&self, abs_path: &str) -> Result<FileEntry, rusqlite::Error> {
        use std::fs;

        // Canonicalize the path (e.g., resolve ./foo/../bar)
        let canonical = fs::canonicalize(abs_path).map_err(|_| rusqlite::Error::InvalidQuery)?; // you can improve this error later

        let abs = canonical.to_string_lossy();

        // Check if the file is already in the DB
        if let Some(existing) = self.get_file_by_path(&abs)? {
            return Ok(existing);
        }

        // Get metadata for insert
        let metadata = fs::metadata(&canonical).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let name = canonical
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string();
        let size_u = metadata.len();
        let size_bytes = i64::try_from(size_u).map_err(|_| rusqlite::Error::InvalidQuery)?; // This file is WAY too large
        let id = uuid::Uuid::new_v4().to_string();

        self.con.execute(
            "INSERT INTO file (id, abs_path, name, size_bytes)
         VALUES (?1, ?2, ?3, ?4)",
            params![id, abs.as_ref(), name, size_bytes],
        )?;

        // Get created_at so the struct is complete
        let created_at: String = self.con.query_row(
            "SELECT created_at FROM file WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        Ok(FileEntry {
            id,
            abs_path: abs.into_owned(),
            name,
            size_bytes,
            created_at,
        })
    }

    /// # Errors
    ///
    /// erroring only if no file or filaure to unpack data
    pub fn get_file_by_path(&self, abs_path: &str) -> Result<Option<FileEntry>, rusqlite::Error> {
        self.con
            .query_row(
                "SELECT id, abs_path, name, size_bytes, created_at FROM file WHERE abs_path = ?1",
                params![abs_path],
                |r| {
                    Ok(FileEntry {
                        id: r.get(0)?,
                        abs_path: r.get(1)?,
                        name: r.get(2)?,
                        size_bytes: r.get(3)?,
                        created_at: r.get(4)?,
                    })
                },
            )
            .optional()
    }

    /// # Errors
    ///
    /// Will error if unable to delete file or file doesn't exist
    /// FK inside shares is set to CASCADE, so no error there
    pub fn delete_file(&self, file_id: &str) -> Result<bool, rusqlite::Error> {
        let changed = self
            .con
            .execute("DELETE FROM file WHERE id = ?1", params![file_id])?;
        Ok(changed > 0)
    }

    // ————— share CRUD (Admin) —————

    /// # Errors
    ///
    /// Fails only with generic db failure to read
    pub fn list_shares(&self) -> Result<Vec<Share>, rusqlite::Error> {
        let mut stmt = self.con.prepare(
            "SELECT slug, file_id, expires_at, max_downloads, dl_count, password_hash, created_at
             FROM share ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Share {
                slug: r.get(0)?,
                file_id: r.get(1)?,
                expires_at: r.get(2)?,
                max_downloads: r.get(3)?,
                dl_count: r.get(4)?,
                password_hash: r.get(5)?,
                created_at: r.get(6)?,
            })
        })?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// # Errors
    ///
    /// Returning errors if data can't be unpacked or share doesn't exist
    pub fn get_share(&self, slug: &str) -> Result<Option<Share>, rusqlite::Error> {
        self.con
            .query_one(
                "
            SELECT slug, file_id, expires_at, max_downloads, dl_count, password_hash, created_at
            FROM share WHERE slug = ?1",
                params![slug],
                |r| {
                    Ok(Share {
                        slug: r.get(0)?,
                        file_id: r.get(1)?,
                        expires_at: r.get(2)?,
                        max_downloads: r.get(3)?,
                        dl_count: r.get(4)?,
                        password_hash: r.get(5)?,
                        created_at: r.get(6)?,
                    })
                },
            )
            .optional()
    }

    /// # Errors
    ///
    /// Can fail if random generation of slugs fails 5 times
    /// other than that, simple read-write server issues or missing file
    pub fn create_share(&self, new_share: &Share) -> Result<Share, rusqlite::Error> {
        // Check if file exists
        let file_exists: bool = self.con.query_one(
            "SELECT EXISTS(SELECT 1 FROM file WHERE id = ?1)",
            params![new_share.file_id],
            |r| r.get(0),
        )?;
        if !file_exists {
            return Err(rusqlite::Error::InvalidQuery);
        }

        let mut slug = gen_slug(SLUG_SIZE);
        let mut attempts = 0;
        // check for slug on DB
        loop {
            let slug_exists: bool = self.con.query_one(
                "SELECT EXISTS(SELECT 1 FROM share WHERE slug = ?1)",
                params![slug],
                |r| r.get(0),
            )?;
            if !slug_exists {
                break;
            }
            slug = gen_slug(SLUG_SIZE);
            attempts += 1;

            if attempts > 5 {
                return Err(rusqlite::Error::ExecuteReturnedResults);
            }
        }

        // Add to db
        let res = self.con.execute(
            "INSERT INTO share (slug, file_id, expires_at, max_downloads, password_hash)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                slug,
                new_share.file_id,
                new_share.expires_at,
                new_share.max_downloads,
                new_share.password_hash
            ],
        );

        match res {
            Ok(_) => self.get_share(&slug)?.ok_or(rusqlite::Error::InvalidQuery),
            Err(e) => Err(e),
        }
    }

    /// # Errors
    ///
    /// Will error if unable to delete share or share doesn't exist
    pub fn delete_share(&self, slug: &str) -> Result<bool, rusqlite::Error> {
        let changed = self
            .con
            .execute("DELETE FROM share WHERE slug = ?1", params![slug])?;
        Ok(changed > 0)
    }

    // ————— share CRUD (User) —————

    ///
    pub fn get_public_info(&self, slug: &str) -> Result<Option<PublicShare>, rusqlite::Error> {
        self.con
            .query_row(
                "
            SELECT
                s.slug,
                f.name,
                f.size_bytes,
                f.created_at,
                s.dl_count,
                s.max_downloads,
                s.expires_at,
                s.password_hash IS NOT NULL
            FROM share s
            JOIN file f ON s.file_id = f.id
            WHERE s.slug = ?1
            ",
                params![slug],
                |r| {
                    Ok(PublicShare {
                        slug: r.get(0)?,
                        file_name: r.get(1)?,
                        file_size: r.get(2)?,
                        created_at: r.get(3)?,
                        dl_count: r.get(4)?,
                        max_downloads: r.get(5)?,
                        expires_at: r.get(6)?,
                        password_required: r.get(7)?,
                    })
                },
            )
            .optional()
    }

    /// #Errors
    ///
    /// Will error if the slug is false
    pub fn check_password(&self, slug: &str, input: &str) -> Result<bool, rusqlite::Error> {
        let share = self.get_share(slug)?.ok_or(rusqlite::Error::InvalidQuery)?;

        match &share.password_hash {
            None => Ok(true),
            Some(hash) => Ok(verify_password(input, hash)),
        }
    }
}
