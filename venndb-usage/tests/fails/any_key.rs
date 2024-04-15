use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(any, key)]
    country: String,
}

fn main() {}
