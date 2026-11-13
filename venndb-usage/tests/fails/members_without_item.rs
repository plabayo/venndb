use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(filter, members = bits)]
    tags: u8,
}

fn main() {}
