use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(set)]
    tags: Vec<String>,
}

fn main() {}
