use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    id: u32,
    name: String,
    is_manager: bool,
    is_admin: bool,
    is_active: bool,
    #[venndb(filter)]
    department: Department,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Department {
    Engineering,
    Sales,
    Marketing,
    HR,
}

fn main() {
    let _ = EmployeeDB::new();
}
