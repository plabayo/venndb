use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(filter, any)]
    is_alive: bool,
}

fn main() {}
