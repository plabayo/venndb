use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(any)]
    is_alive: bool,
}

fn main() {}
