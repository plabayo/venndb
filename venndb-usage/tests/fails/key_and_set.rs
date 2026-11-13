use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(key, set)]
    tags: Vec<String>,
}

fn main() {}
