use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(foo)]
    id: u32,
}

fn main() {}
