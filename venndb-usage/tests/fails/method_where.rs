use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(filter)]
    tag: String,
    tag_where: bool,
}

fn main() {}
