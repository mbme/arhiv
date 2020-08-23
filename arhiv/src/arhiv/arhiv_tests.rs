use crate::{Arhiv, ArhivNotes, Config};
use anyhow::*;
use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

impl Drop for Arhiv {
    // teardown
    fn drop(&mut self) {
        println!(
            "DROPPING {} at {}",
            if self.config.is_prime {
                "PRIME"
            } else {
                "REPLICA"
            },
            self.get_root_dir()
        );
        fs::remove_dir_all(self.get_root_dir()).expect("must be able to remove arhiv");
    }
}

pub fn new_arhiv(prime: bool) -> Arhiv {
    let server_port = 9876;

    let primary_url = {
        if prime {
            None
        } else {
            Some(format!("http://localhost:{}", server_port))
        }
    };

    let config = Config {
        is_prime: prime,
        arhiv_root: generate_temp_dir("TempArhiv"),
        primary_url,
        server_port,
    };

    Arhiv::create(config).expect("must be able to create temp arhiv")
}

fn generate_temp_dir(prefix: &str) -> String {
    let mut path = env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();

    path.push(format!("{}-{}", prefix, nanos));

    path.to_str()
        .expect("must be able to convert path to string")
        .to_string()
}

#[test]
fn it_works() -> Result<()> {
    let arhiv = new_arhiv(true);
    assert_eq!(arhiv.list_documents(None)?.len(), 0);

    Ok(())
}

fn test_crud(arhiv: &Arhiv) -> Result<()> {
    // CREATE
    let mut document = ArhivNotes::create_note();
    document.data = ArhivNotes::data("test", "test");
    arhiv.stage_document(document.clone())?;
    assert_eq!(arhiv.list_documents(None)?.len(), 1);

    // READ
    {
        let other_document = arhiv.get_document(&document.id)?.unwrap();

        assert_eq!(other_document.data, document.data);
        assert_eq!(other_document.is_staged(), true);
    }

    // UPDATE
    {
        let mut other_document = arhiv.get_document(&document.id)?.unwrap();
        other_document.data = ArhivNotes::data("1", "1");
        arhiv.stage_document(other_document.clone())?;

        assert_eq!(
            arhiv.get_document(&document.id)?.unwrap().data,
            other_document.data
        );
    }

    // DELETE
    {
        assert_eq!(arhiv.list_documents(None)?.len(), 1);
        let mut other_document = arhiv.get_document(&document.id)?.unwrap();
        other_document.archived = true;
        arhiv.stage_document(other_document)?;

        assert_eq!(arhiv.get_document(&document.id)?.unwrap().archived, true);
        assert_eq!(arhiv.list_documents(None)?.len(), 0);
    }

    Ok(())
}

#[test]
fn test_prime_crud() -> Result<()> {
    test_crud(&new_arhiv(true))
}

#[test]
fn test_replica_crud() -> Result<()> {
    test_crud(&new_arhiv(false))
}

#[tokio::test]
async fn test_prime_sync() -> Result<()> {
    let arhiv = new_arhiv(true);

    let document = ArhivNotes::create_note();
    arhiv.stage_document(document.clone())?;
    assert_eq!(arhiv.get_document(&document.id)?.unwrap().is_staged(), true);

    arhiv.sync().await?;

    assert_eq!(
        arhiv.get_document(&document.id)?.unwrap().is_staged(),
        false
    );

    Ok(())
}

#[tokio::test]
async fn test_replica_sync() -> Result<()> {
    let prime = new_arhiv(true);
    let (join_handle, shutdown_sender) = prime.start_server();

    let replica = new_arhiv(false);
    let document = ArhivNotes::create_note();
    replica.stage_document(document.clone())?;

    replica.sync().await?;

    assert_eq!(
        replica.get_document(&document.id)?.unwrap().is_staged(),
        false
    );

    shutdown_sender.send(()).unwrap();
    join_handle.await.unwrap();

    Ok(())
}
