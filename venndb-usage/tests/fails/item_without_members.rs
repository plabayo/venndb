use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(filter, set, item = String)]
    tags: Vec<String>,
}

fn main() {}
