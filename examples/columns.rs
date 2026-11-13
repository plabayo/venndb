//! Filters on what is no property of a row, or changes while the database is in use.
//!
//! Run with `cargo run --example columns`, and see the `runtime` example
//! for more on the columns that change.

use venndb::{Filter, RuntimeColumn, VennDB};

#[derive(Debug, VennDB)]
pub struct Employee {
    #[venndb(key)]
    id: u32,
    years: u32,
    is_manager: bool,
}

fn ids(result: Option<EmployeeDBQueryResult<'_>>) -> Vec<u32> {
    result
        .map(|result| result.iter().map(|e| e.id).collect())
        .unwrap_or_default()
}

fn main() {
    let db = EmployeeDB::from_iter((1..=60).map(|id| Employee {
        id,
        years: id % 20,
        is_manager: id % 7 == 0,
    }))
    .unwrap();

    // computed once, for a fact about a row that is no property
    let seniors = db.column(|employee| employee.years > 10);
    // set and unset at any time and from any thread, without a lock
    let on_leave = RuntimeColumn::new();
    on_leave.set(db.index_by_id(&35).unwrap(), true);

    let mut query = db.query();
    query
        .is_manager(true)
        .and(Filter::Is(&seniors))
        .and(Filter::Runtime(&on_leave, false));
    assert_eq!(ids(query.execute()), [14, 56]);

    // back from leave: the same query now finds 35 as well
    on_leave.set(db.index_by_id(&35).unwrap(), false);
    assert_eq!(ids(query.execute()), [14, 35, 56]);

    // a row can be claimed by one thread only
    let busy = RuntimeColumn::new();
    let claims: usize = std::thread::scope(|scope| {
        let claim = || usize::from(busy.claim(db.index_by_id(&14).unwrap()));
        let threads: Vec<_> = (0..4).map(|_| scope.spawn(claim)).collect();
        threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .sum()
    });
    assert_eq!(claims, 1);
    query.and(Filter::Runtime(&busy, false));
    assert_eq!(ids(query.execute()), [35, 56]);

    // for a condition that fits no column: a random row for which it holds
    let found = query.execute().unwrap();
    let employee = found.any_where(|employee| employee.id > 40).unwrap();
    assert_eq!(employee.id, 56);
    assert!(found.any_where(|employee| employee.id > 60).is_none());
}

#[test]
fn runs() {
    main();
}
