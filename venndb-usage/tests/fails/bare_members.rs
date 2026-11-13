use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(filter, members)]
    tags: Vec<String>,
}

fn main() {}
