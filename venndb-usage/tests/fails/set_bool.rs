use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(set)]
    is_alive: bool,
}

fn main() {}
