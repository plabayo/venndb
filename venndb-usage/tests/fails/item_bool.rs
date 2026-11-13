use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(item = "u8")]
    level: bool,
}

fn main() {}
