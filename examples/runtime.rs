//! A column that changes while the database is in use: who sets it, who reads it,
//! and what becomes of it when rows are added or the database is replaced.
//!
//! Run with `cargo run --example runtime`.

use std::sync::Arc;
use venndb::{Filter, RuntimeColumn, VennDB};

#[derive(Debug, Clone, VennDB)]
pub struct Worker {
    #[venndb(key)]
    name: String,
    #[venndb(filter)]
    region: &'static str,
}

fn worker(name: &str, region: &'static str) -> Worker {
    Worker {
        name: name.to_owned(),
        region,
    }
}

/// The workers of a region that are not blocked.
fn available(db: &WorkerDB, blocked: &RuntimeColumn, region: &'static str) -> Vec<String> {
    let mut query = db.query();
    query.region(region).and(Filter::Runtime(blocked, false));
    query
        .execute()
        .map(|found| found.iter().map(|worker| worker.name.clone()).collect())
        .unwrap_or_default()
}

/// The column for a new database, with the rows set that were set for the old one.
///
/// A column knows rows by their position, which is not the same in another database:
/// the key of a row is what tells which row of the new database it is.
fn carry_over(old: &WorkerDB, blocked: &RuntimeColumn, new: &WorkerDB) -> RuntimeColumn {
    let column = RuntimeColumn::new();
    for worker in blocked.iter().filter_map(|index| old.get(index)) {
        if let Some(index) = new.index_by_name(&worker.name) {
            column.set(index, true);
        }
    }
    column
}

fn main() {
    let db = Arc::new(
        WorkerDB::from_rows(vec![
            worker("ada", "eu"),
            worker("bob", "eu"),
            worker("cyd", "us"),
            worker("dee", "eu"),
        ])
        .unwrap(),
    );

    // a column is no part of a database: it is told which rows are set, by their index
    let blocked = RuntimeColumn::new();
    assert_eq!(blocked.count(), 0);
    assert_eq!(available(&db, &blocked, "eu"), ["ada", "bob", "dee"]);

    // a clone is the same column: hand one to whatever knows when a row has to change,
    // it needs no lock and no mutable access to the database
    let reporter = {
        let (db, blocked) = (db.clone(), blocked.clone());
        std::thread::spawn(move || {
            for name in ["bob", "cyd"] {
                blocked.set(db.index_by_name(name).unwrap(), true);
            }
        })
    };
    reporter.join().unwrap();

    // every query from then on sees it, at the cost of one more filter
    assert_eq!(available(&db, &blocked, "eu"), ["ada", "dee"]);
    assert_eq!(available(&db, &blocked, "us"), [] as [&str; 0]);
    assert!(blocked.get(db.index_by_name("bob").unwrap()));
    assert_eq!(blocked.iter().collect::<Vec<_>>(), [1, 2]);

    // `set` returns what the row was before, `claim` if it was this call that set it
    assert!(blocked.set(db.index_by_name("cyd").unwrap(), false));
    assert!(blocked.claim(db.index_by_name("cyd").unwrap()));
    assert!(!blocked.claim(db.index_by_name("cyd").unwrap()));

    // rows appended afterwards are not set, until they are: the column grows as needed
    let mut grown = WorkerDB::from_rows(db.iter().cloned().collect()).unwrap();
    grown.append(worker("eve", "eu")).unwrap();
    assert_eq!(available(&grown, &blocked, "eu"), ["ada", "dee", "eve"]);
    blocked.set(grown.index_by_name("eve").unwrap(), true);
    assert_eq!(available(&grown, &blocked, "eu"), ["ada", "dee"]);
    blocked.set(grown.index_by_name("eve").unwrap(), false);

    // a database that replaces this one has its rows elsewhere: make a column for it
    let reloaded = WorkerDB::from_rows(vec![
        worker("eve", "eu"),
        worker("bob", "eu"),
        worker("ada", "eu"),
    ])
    .unwrap();
    let blocked = carry_over(&db, &blocked, &reloaded);
    assert_eq!(blocked.count(), 1);
    // bob is still blocked, cyd is gone and eve is new
    assert_eq!(available(&reloaded, &blocked, "eu"), ["eve", "ada"]);

    blocked.clear();
    assert_eq!(available(&reloaded, &blocked, "eu"), ["eve", "bob", "ada"]);
}

#[test]
fn runs() {
    main();
}
