#[macro_use]
extern crate rental;

use std::time::Duration;

use assert_matches::assert_matches;
use rmpv::Value;
use serde::{Deserialize, Serialize};
use tarantool_rs::{errors::Error, Connection, Executor, ExecutorExt};

use crate::common::TarantoolTestContainer;

mod common;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct CrewMember {
    id: u32,
    name: String,
    rank: String,
    occupation: String,
}

#[tokio::test]
async fn image_test() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    let conn = container.create_conn().await?;
    conn.ping().await?;

    Ok(())
}

#[tokio::test]
async fn auth_ok() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    let conn = Connection::builder()
        .auth("Sisko", Some("A-4-7-1"))
        .build(format!("127.0.0.1:{}", container.connect_port()))
        .await?;
    conn.ping().await?;
    Ok(())
}

#[tokio::test]
async fn auth_err() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    assert_matches!(
        Connection::builder()
            .auth("Quark", Some("Q-0-0-0"))
            .build(format!("127.0.0.1:{}", container.connect_port()))
            .await
            .map(drop),
        Err(Error::Auth(_))
    );

    Ok(())
}

#[tokio::test]
async fn eval() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    let conn = container.create_conn().await?;
    let res: u32 = conn.eval("return ...", (42,)).await?.decode_result()?;
    assert_eq!(res, 42);

    Ok(())
}

#[tokio::test]
async fn call() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    let conn = container.create_conn().await?;
    let res: String = conn.call("station_name", (false,)).await?.decode_first()?;
    assert_eq!(res, "Deep Space 9");

    Ok(())
}

#[tokio::test]
async fn retrieve_schema() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    let conn = container.create_conn().await?;
    let space = conn
        .space("ds9_crew")
        .await?
        .expect("Space 'ds9_crew' found");
    assert_eq!(
        space.metadata().id(),
        512,
        "First user space expected to have id 512"
    );
    assert_eq!(space.metadata().name(), "ds9_crew");

    let index_count = space.indices().count();
    assert!(index_count > 1, "There should be multiple indices");

    Ok(())
}

#[tokio::test]
async fn select_all() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    let conn: Connection = container.create_conn().await?;
    let space = conn
        .space("ds9_crew")
        .await?
        .expect("Space 'ds9_crew' found");

    let members: Vec<CrewMember> = space
        .select(None, None, Some(tarantool_rs::IteratorType::All), ())
        .await?;
    assert_eq!(members.len(), 7);
    assert_eq!(
        members[1],
        CrewMember {
            id: 2,
            name: "Kira Nerys".into(),
            rank: "Major".into(),
            occupation: "First officer".into()
        }
    );

    Ok(())
}

#[tokio::test]
async fn select_limits() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    let conn: Connection = container.create_conn().await?;
    let space = conn
        .space("ds9_crew")
        .await?
        .expect("Space 'ds9_crew' found");

    let members: Vec<CrewMember> = space
        .select(Some(2), Some(2), Some(tarantool_rs::IteratorType::All), ())
        .await?;
    assert_eq!(members.len(), 2);
    assert_eq!(
        members[1],
        CrewMember {
            id: 4,
            name: "Julian Bashir".into(),
            rank: "Lieutenant".into(),
            occupation: "Chief medical officer".into()
        }
    );

    Ok(())
}

#[tokio::test]
async fn select_by_key() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    let conn: Connection = container.create_conn().await?;
    let space = conn
        .space("ds9_crew")
        .await?
        .expect("Space 'ds9_crew' found");
    let rank_idx = space.index("idx_rank").expect("Rank index present");

    let members: Vec<CrewMember> = rank_idx
        .select(None, None, None, ("Lieutenant Commander",))
        .await?;
    assert_eq!(members.len(), 2);
    assert_eq!(
        members[0],
        CrewMember {
            id: 3,
            name: "Jadzia Dax".into(),
            rank: "Lieutenant Commander".into(),
            occupation: "Science officer".into()
        }
    );

    Ok(())
}

#[tokio::test]
async fn timeout() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    let conn = Connection::builder()
        .timeout(Duration::from_millis(100))
        .build(format!("127.0.0.1:{}", container.connect_port()))
        .await?;

    assert_matches!(
        conn.eval("require('fiber').sleep(1)", ()).await,
        Err(tarantool_rs::Error::Timeout)
    );

    Ok(())
}

#[tokio::test]
async fn dmo() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    let conn = Connection::builder()
        .timeout(Duration::from_millis(100))
        .build(format!("127.0.0.1:{}", container.connect_port()))
        .await?;

    let tx = conn.transaction().await?;
    let space = tx.space("ds9_crew").await?.expect("Space 'ds9_crew' found");
    let name_idx = space.index("idx_name").unwrap();

    // update
    let new_value: CrewMember = name_idx
        .update(
            ("Benjamin Sisko",),
            (Value::Array(vec!["=".into(), 2.into(), "Captain".into()]),),
        )
        .await?
        .decode()?;
    assert_eq!(
        new_value,
        CrewMember {
            id: 1,
            name: "Benjamin Sisko".into(),
            rank: "Captain".into(),
            occupation: "Commanding officer".into()
        }
    );

    // delete
    let _: CrewMember = name_idx.delete(("Jadzia Dax",)).await?.decode()?;

    // insert
    let _: CrewMember = space
        .insert((None::<()>, "Ezri Dax", "Ensign", "Counselor"))
        .await?
        .decode()?;

    tx.commit().await?;

    Ok(())
}
