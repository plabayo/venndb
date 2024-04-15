use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(any)]
    country: String,
}

fn main() {}
