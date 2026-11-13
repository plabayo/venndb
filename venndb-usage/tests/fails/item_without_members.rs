use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(filter, set, item = String)]
    tags: std::collections::BTreeSet<String>,
}

fn main() {}
