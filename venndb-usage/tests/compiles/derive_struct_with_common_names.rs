//! Functions and fields named like the variables of the generated code.

use venndb::VennDB;

fn value(value: &String) -> bool {
    value == "*"
}

fn data(row: &Row) -> u32 {
    row.index
}

fn member(members: &Vec<u8>) -> impl Iterator<Item = &u8> {
    members.iter()
}

fn index(row: &Row) -> bool {
    row.index > 0
}

#[derive(Debug, VennDB)]
#[venndb(validator = index)]
struct Row {
    #[venndb(key = data)]
    index: u32,
    #[venndb(filter, any = value)]
    value: String,
    #[venndb(filter, any = value)]
    any: Option<String>,
    #[venndb(filter, members = member)]
    member: Vec<u8>,
    rows: bool,
    db: bool,
    plan: bool,
    len: bool,
    get: bool,
    column: bool,
    #[venndb(filter)]
    key: u8,
    predicate: bool,
}

fn main() {
    let mut db = RowDB::new();
    db.append(Row {
        index: 1,
        value: "*".to_owned(),
        any: None,
        member: vec![1],
        rows: true,
        db: true,
        plan: true,
        len: true,
        get: true,
        column: true,
        key: 1,
        predicate: true,
    })
    .unwrap();
    let mut query = db.query();
    query
        .value("a")
        .member(1)
        .rows(true)
        .db(true)
        .plan(true)
        .len(true)
        .get(true)
        .column(true)
        .key(1)
        .predicate(true);
    assert_eq!(query.execute().unwrap().first().index, 1);
    assert!(query.any("b").execute().is_none());
}
