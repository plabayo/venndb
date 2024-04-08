use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    id: u32,
    is_manager: bool,
    is_active: bool,
    #[venndb(filter, key)]
    country: String,
}

fn main() {}
