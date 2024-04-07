use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(key)]
    id: u32,
    name: String,
    is_manager: bool,
    is_admin: bool,
    is_active: bool,
    department: Department,
}

#[derive(Debug)]
pub enum Department {
    Engineering,
    Sales,
    Marketing,
    HR,
}

fn main() {
    let _ = EmployeeDB::new();
}
