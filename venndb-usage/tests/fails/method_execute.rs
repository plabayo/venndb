use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(filter)]
    execute: String,
}

fn main() {}
