use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(skip)]
    id: u32,
    #[venndb(skip)]
    name: String,
    #[venndb(skip)]
    is_manager: bool,
    #[venndb(skip)]
    department: Department,
}

#[derive(Debug)]
pub enum Department {
    Engineering,
    Sales,
    Marketing,
    HR,
}

fn main() {}
