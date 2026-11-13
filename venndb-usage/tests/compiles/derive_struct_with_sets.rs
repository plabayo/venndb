use std::collections::BTreeSet;
use venndb::{Filter, VennDB};

fn is_any(value: &u8) -> bool {
    *value == 0
}

fn code(employee: &Employee) -> String {
    employee.id.to_string()
}

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(key)]
    id: u32,
    #[venndb(key = code)]
    code: String,
    #[venndb(filter, set)]
    tags: Vec<String>,
    #[venndb(filter, members = BTreeSet::iter)]
    floors: Option<BTreeSet<u8>>,
    #[venndb(filter, members = <[u16]>::iter, item = "u16")]
    rooms: Box<[u16]>,
    #[venndb(filter, any = is_any)]
    level: u8,
}

fn main() {
    let db = EmployeeDB::new();
    let active = db.runtime_column();
    let senior = db.column(|employee| employee.level > 3);
    let mut query = db.query();
    query
        .tags("a")
        .floors(1)
        .rooms(2u16)
        .level(3)
        .tags_where(|tag| tag.is_empty())
        .and(Filter::Runtime(&active, true))
        .and(Filter::Not(&senior));
    assert!(query.execute().is_none());
    assert!(db.index_by_code("1").is_none());
}
