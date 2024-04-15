use venndb::{Any, VennDB};

#[derive(Debug, VennDB)]
struct Employee {
    id: u32,
    name: String,
    is_manager: bool,
    is_admin: bool,
    is_active: bool,
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

impl Any for Department {
    fn is_any(&self) -> bool {
        self == &Department::Any
    }
}

fn main() {
    let _ = EmployeeDB::new();
}
