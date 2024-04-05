use venndb::VennDB;

#[derive(Debug, VennDB)]
#[venndb(foo)]
struct Employee {
    id: u32,
}

fn main() {}
