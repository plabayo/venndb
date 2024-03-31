use venndb::VennDB;

#[derive(Debug, VennDB)]
#[venndb(name = "Database")]
struct Employee {
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
    let _ = Database::new();
}