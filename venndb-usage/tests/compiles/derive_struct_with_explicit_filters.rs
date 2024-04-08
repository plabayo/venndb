use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(key)]
    id: u32,
    name: String,
    #[venndb(filter)]
    is_manager: bool,
    #[venndb(filter)]
    is_admin: bool,
    #[venndb(filter)]
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
