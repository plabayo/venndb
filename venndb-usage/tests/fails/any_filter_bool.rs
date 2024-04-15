use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(any, filter)]
    is_alive: bool,
}

fn main() {}
