use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(filter, set)]
    tags: Vec<String>,
}

fn main() {}
