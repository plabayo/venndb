use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(filter, set, members = BTreeSet::iter)]
    tags: std::collections::BTreeSet<String>,
}

fn main() {}
