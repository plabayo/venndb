use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(filter)]
    tag: String,
    r#tag_where: bool,
}

fn main() {}
