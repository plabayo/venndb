use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    any: bool,
    #[venndb(filter)]
    tag: String,
    tag_ref: bool,
}

fn main() {}
