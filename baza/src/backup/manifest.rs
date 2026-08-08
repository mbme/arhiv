use std::{
    collections::HashSet,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

use baza_common::{
    ExposeSecret, FsTransaction, SecretString, bytes_to_hex_string, create_file_reader,
    file_exists, get_crate_version, get_file_hash_sha256, path_file_name, path_to_string,
    relative_path,
};
use baza_storage::crypto::age::{AgeKey, encrypt_and_write, read_and_decrypt_file};

use crate::entities::Id;

const MANIFEST_VERSION: u8 = 1;
pub(crate) const MANIFEST_SUFFIX: &str = ".manifest.age";
const KEY_SUFFIX: &str = ".key.age";

/// Inputs needed to bind one backup generation into an encrypted manifest.
pub(crate) struct BackupManifestWrite<'a> {
    pub(crate) backup_dir: &'a str,
    pub(crate) timestamp: &'a str,
    pub(crate) key_path: &'a str,
    pub(crate) db_path: &'a str,
    pub(crate) storage_key: AgeKey,
    pub(crate) blob_ids: Vec<Id>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct BackupManifest {
    version: u8,
    timestamp: String,
    tool_version: String,
    key: BackupArtifact,
    db: BackupArtifact,
    blobs: Vec<BackupBlobArtifact>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct BackupArtifact {
    path: String,
    ciphertext_sha256: String,
}

/// Manifest entry for a backed-up encrypted asset blob.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackupBlobArtifact {
    pub(crate) id: Id,
    path: String,
    ciphertext_sha256: String,
}

/// Loaded backup generation with manifest-relative path and artifact verification rules.
pub(crate) struct BackupBundle {
    manifest_path: PathBuf,
    manifest: BackupManifest,
    storage_key: AgeKey,
}

impl BackupBundle {
    /// Decrypts the same-generation key artifact, then loads and validates the manifest.
    pub(crate) fn load(manifest_path: &str, password: SecretString) -> Result<Self> {
        let manifest_path_buf = PathBuf::from(manifest_path);
        let key_path = manifest_to_key_path(&manifest_path_buf)?;
        let storage_key = decrypt_backup_key(&key_path, password)?;
        let manifest = read_manifest_file(&manifest_path_buf, storage_key.clone())?;

        validate_manifest(&manifest)?;
        ensure!(
            manifest.key.path == path_file_name(&key_path)?,
            "Manifest key artifact doesn't match manifest path convention"
        );

        Ok(Self {
            manifest_path: manifest_path_buf,
            manifest,
            storage_key,
        })
    }

    /// Returns the timestamp recorded by the manifest.
    pub(crate) fn timestamp(&self) -> &str {
        &self.manifest.timestamp
    }

    /// Returns the storage key recovered from this backup generation's key artifact.
    pub(crate) fn storage_key(&self) -> &AgeKey {
        &self.storage_key
    }

    /// Resolves the backed-up DB artifact path.
    pub(crate) fn db_path(&self) -> Result<PathBuf> {
        self.artifact_path(&self.manifest.db)
    }

    /// Returns the blob artifacts recorded in manifest order.
    pub(crate) fn blobs(&self) -> &[BackupBlobArtifact] {
        &self.manifest.blobs
    }

    /// Resolves a blob artifact path relative to the manifest directory.
    pub(crate) fn blob_path(&self, blob: &BackupBlobArtifact) -> Result<PathBuf> {
        self.artifact_path(&blob.artifact())
    }

    /// Verifies key and DB ciphertext hashes against the manifest.
    pub(crate) fn verify_key_and_db(&self) -> Result<()> {
        self.verify_artifact(&self.manifest.key)?;
        self.verify_artifact(&self.manifest.db)
    }

    /// Verifies a blob ciphertext hash against the manifest.
    pub(crate) fn verify_blob(&self, blob: &BackupBlobArtifact) -> Result<()> {
        self.verify_artifact(&blob.artifact())
    }

    /// Checks whether a blob artifact exists at its manifest-relative path.
    pub(crate) fn blob_exists(&self, blob: &BackupBlobArtifact) -> Result<bool> {
        file_exists(&path_to_string(self.blob_path(blob)?))
    }

    /// Copies the key artifact and verifies copied bytes before transaction commit.
    pub(crate) fn copy_key_to(&self, fs_tx: &mut FsTransaction, dest: &str) -> Result<()> {
        self.copy_artifact(fs_tx, &self.manifest.key, dest)
    }

    /// Copies the DB artifact and verifies copied bytes before transaction commit.
    pub(crate) fn copy_db_to(&self, fs_tx: &mut FsTransaction, dest: &str) -> Result<()> {
        self.copy_artifact(fs_tx, &self.manifest.db, dest)
    }

    /// Copies a blob artifact and verifies copied bytes before transaction commit.
    pub(crate) fn copy_blob_to(
        &self,
        fs_tx: &mut FsTransaction,
        blob: &BackupBlobArtifact,
        dest: &str,
    ) -> Result<()> {
        self.copy_artifact(fs_tx, &blob.artifact(), dest)
    }

    fn artifact_path(&self, artifact: &BackupArtifact) -> Result<PathBuf> {
        Ok(self.base_dir()?.join(&artifact.path))
    }

    fn base_dir(&self) -> Result<PathBuf> {
        Ok(self
            .manifest_path
            .parent()
            .context("Manifest path must have a parent directory")?
            .to_path_buf())
    }

    fn verify_artifact(&self, artifact: &BackupArtifact) -> Result<()> {
        let path = self.artifact_path(artifact)?;
        ensure!(
            file_exists(&path_to_string(&path))?,
            "Backup artifact {} is missing",
            artifact.path
        );

        let actual = file_sha256_hex(&path_to_string(&path))?;
        ensure!(
            actual == artifact.ciphertext_sha256,
            "Backup artifact {} hash mismatch",
            artifact.path
        );

        Ok(())
    }

    fn copy_artifact(
        &self,
        fs_tx: &mut FsTransaction,
        artifact: &BackupArtifact,
        dest: &str,
    ) -> Result<()> {
        let src = self.artifact_path(artifact)?;
        fs_tx.copy_file(path_to_string(src), dest)?;

        let actual = file_sha256_hex(dest)?;
        ensure!(
            actual == artifact.ciphertext_sha256,
            "Restored artifact {} hash mismatch",
            artifact.path
        );

        Ok(())
    }
}

impl BackupBlobArtifact {
    fn artifact(&self) -> BackupArtifact {
        BackupArtifact {
            path: self.path.clone(),
            ciphertext_sha256: self.ciphertext_sha256.clone(),
        }
    }
}

/// Writes the encrypted manifest that binds one backup generation's artifact bytes.
pub(crate) fn write_backup_manifest(
    fs_tx: &mut FsTransaction,
    request: BackupManifestWrite<'_>,
) -> Result<String> {
    let BackupManifestWrite {
        backup_dir,
        timestamp,
        key_path,
        db_path,
        storage_key,
        mut blob_ids,
    } = request;

    blob_ids.sort_by_key(ToString::to_string);
    let mut blobs = Vec::new();
    for blob_id in &blob_ids {
        let blob_path = format!("{backup_dir}/data/{blob_id}.age");
        if file_exists(&blob_path)? {
            blobs.push(BackupBlobArtifact {
                id: blob_id.clone(),
                path: relative_path(backup_dir, &blob_path)?,
                ciphertext_sha256: file_sha256_hex(&blob_path)?,
            });
        }
    }

    let manifest = BackupManifest {
        version: MANIFEST_VERSION,
        timestamp: timestamp.to_string(),
        tool_version: get_crate_version().to_string(),
        key: BackupArtifact {
            path: relative_path(backup_dir, key_path)?,
            ciphertext_sha256: file_sha256_hex(key_path)?,
        },
        db: BackupArtifact {
            path: relative_path(backup_dir, db_path)?,
            ciphertext_sha256: file_sha256_hex(db_path)?,
        },
        blobs,
    };

    let manifest_path = format!("{backup_dir}/{timestamp}{MANIFEST_SUFFIX}");
    let manifest_json = serde_json::to_vec(&manifest)?;
    let manifest_ciphertext = encrypt_and_write(Vec::new(), storage_key, &manifest_json, false)?;
    fs_tx.create_file(&manifest_path, &manifest_ciphertext)?;

    Ok(manifest_path)
}

fn file_sha256_hex(path: &str) -> Result<String> {
    let hash = get_file_hash_sha256(create_file_reader(path)?)?;
    Ok(bytes_to_hex_string(&hash))
}

fn decrypt_backup_key(key_path: &Path, password: SecretString) -> Result<AgeKey> {
    let mut key_file_key = AgeKey::from_password(password)?;
    if cfg!(test) {
        key_file_key.test_mode();
    }

    let key_data = read_and_decrypt_file(&path_to_string(key_path), key_file_key, true)?;
    let key_data: SecretString = key_data.try_into()?;

    AgeKey::from_age_x25519_key(key_data)
}

fn read_manifest_file(manifest_path: &Path, storage_key: AgeKey) -> Result<BackupManifest> {
    let manifest_data = read_and_decrypt_file(&path_to_string(manifest_path), storage_key, false)?;
    serde_json::from_slice(manifest_data.expose_secret()).context("Failed to parse backup manifest")
}

fn validate_manifest(manifest: &BackupManifest) -> Result<()> {
    ensure!(
        manifest.version == MANIFEST_VERSION,
        "Unsupported backup manifest version {}",
        manifest.version
    );
    validate_manifest_relative_path(&manifest.key.path)?;
    validate_manifest_relative_path(&manifest.db.path)?;

    let mut blob_ids = HashSet::new();
    for blob in &manifest.blobs {
        validate_manifest_relative_path(&blob.path)?;
        ensure!(
            blob_ids.insert(blob.id.clone()),
            "Backup manifest contains duplicate BLOB artifact {}",
            blob.id
        );
    }

    Ok(())
}

fn validate_manifest_relative_path(path: &str) -> Result<()> {
    let path = Path::new(path);
    ensure!(
        !path.is_absolute(),
        "Backup manifest artifact paths must be relative"
    );
    ensure!(
        path.components()
            .all(|component| matches!(component, Component::Normal(_))),
        "Backup manifest artifact paths must stay inside the backup directory"
    );

    Ok(())
}

fn manifest_to_key_path(manifest_path: &Path) -> Result<PathBuf> {
    let manifest = path_to_string(manifest_path);
    let key = manifest
        .strip_suffix(MANIFEST_SUFFIX)
        .context("Manifest path must end with .manifest.age")?;

    Ok(PathBuf::from(format!("{key}{KEY_SUFFIX}")))
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baza_common::{FsTransaction, TempFile};

    use super::*;

    #[test]
    fn copy_artifact_rolls_back_hash_mismatch() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_backup_manifest", "");
        temp_dir.mkdir()?;
        let src = temp_dir.new_child("src");
        src.write_str("tampered")?;
        let expected = temp_dir.new_child("expected");
        expected.write_str("expected")?;
        let dest = temp_dir.new_child("dest");
        dest.write_str("live")?;

        let bundle = BackupBundle {
            manifest_path: PathBuf::from(format!("{}/backup.manifest.age", temp_dir.path)),
            manifest: BackupManifest {
                version: MANIFEST_VERSION,
                timestamp: "test".to_string(),
                tool_version: "test".to_string(),
                key: BackupArtifact {
                    path: "key.age".to_string(),
                    ciphertext_sha256: String::new(),
                },
                db: BackupArtifact {
                    path: "db.age".to_string(),
                    ciphertext_sha256: String::new(),
                },
                blobs: Vec::new(),
            },
            storage_key: AgeKey::from_password("password".into())?,
        };
        let artifact = BackupArtifact {
            path: path_file_name(Path::new(&src.path))?,
            ciphertext_sha256: file_sha256_hex(&expected.path)?,
        };
        let mut fs_tx = FsTransaction::new();

        let err = bundle
            .copy_artifact(&mut fs_tx, &artifact, &dest.path)
            .expect_err("hash mismatch must fail restore copy");
        assert!(format!("{err:?}").contains("hash mismatch"), "{err:?}");

        fs_tx.rollback()?;
        assert_eq!(src.str_contents()?, "tampered");
        assert_eq!(dest.str_contents()?, "live");

        Ok(())
    }
}
