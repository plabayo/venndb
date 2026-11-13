use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(filter, set, members = Vec::iter)]
    tags: Vec<String>,
}

fn main() {}
