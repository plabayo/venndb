use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(filter, any)]
    department: Department,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Department {
    Any,
    Engineering,
    Sales,
    Marketing,
    HR,
}

fn main() {
    let _ = EmployeeDB::new();
}
