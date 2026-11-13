use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(filter, any, set)]
    tags: Vec<String>,
}

fn main() {}
