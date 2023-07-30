#[macro_use]
extern crate rental;

use assert_matches::assert_matches;
use serde::Deserialize;
use tarantool_rs::{errors::Error, schema::SpaceMetadata, Connection, ConnectionLike};

use crate::common::TarantoolTestContainer;

mod common;

#[derive(Debug, Deserialize, PartialEq)]
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
    let (res,): (u32,) = conn.eval("return ...", vec![42.into()]).await?;
    assert_eq!(res, 42);

    Ok(())
}

#[tokio::test]
async fn call() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    let conn = container.create_conn().await?;
    let (res,): (String,) = conn.call("station_name", vec![false.into()]).await?;
    assert_eq!(res, "Deep Space 9");

    Ok(())
}

#[tokio::test]
async fn retrieve_schema() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    let conn = container.create_conn().await?;
    let space = conn
        .load_by_name("ds9_crew")
        .await?
        .expect("Space 'ds9_crew' found");
    assert_eq!(space.id(), 512, "First user space expected to have id 512");
    assert_eq!(space.name(), "ds9_crew");

    assert_eq!(space.indices().len(), 3);
    let primary_idx = space
        .metadata()
        .indices()
        .get_by_id(0)
        .expect("Primary index present");
    assert_eq!(primary_idx.name(), "idx_id");
    assert_eq!(primary_idx.space_id(), 512);
    assert_eq!(primary_idx.id(), 0);

    Ok(())
}

#[tokio::test]
async fn select_all() -> Result<(), anyhow::Error> {
    let container = TarantoolTestContainer::default();

    let conn: Connection = container.create_conn().await?;
    let space = SpaceMetadata::load_by_name(&conn, "ds9_crew")
        .await?
        .expect("Space 'ds9_crew' found");
    let primary_idx = space.indices().get_by_id(0).expect("Primary index present");

    let members: Vec<CrewMember> = conn
        .select(
            space.id(),
            primary_idx.id(),
            None,
            None,
            Some(tarantool_rs::IteratorType::All),
            vec![],
        )
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
    let space = SpaceMetadata::load_by_name(&conn, "ds9_crew")
        .await?
        .expect("Space 'ds9_crew' found");
    let primary_idx = space.indices().get_by_id(0).expect("Primary index present");

    let members: Vec<CrewMember> = conn
        .select(
            space.id(),
            primary_idx.id(),
            Some(2),
            Some(2),
            Some(tarantool_rs::IteratorType::All),
            vec![],
        )
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
    let space = SpaceMetadata::load_by_name(&conn, "ds9_crew")
        .await?
        .expect("Space 'ds9_crew' found");
    let rank_idx = space
        .indices()
        .get_by_name("idx_rank")
        .expect("Rank index present");

    let members: Vec<CrewMember> = conn
        .select(
            space.id(),
            rank_idx.id(),
            None,
            None,
            None,
            vec!["Lieutenant Commander".into()],
        )
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
