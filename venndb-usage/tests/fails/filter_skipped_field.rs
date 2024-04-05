use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    id: u32,
    is_manager: bool,
    #[venndb(skip)]
    is_active: bool,
}

fn main() {
    let mut db = EmployeeDB::new();
    db.append(Employee {
        id: 1,
        is_manager: true,
        is_active: true,
    });

    let mut query = db.query();
    query.is_active(true);
    assert!(query.execute().is_some());
}
