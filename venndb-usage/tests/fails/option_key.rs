use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(key)]
    id: Option<u32>,
}

fn main() {}
