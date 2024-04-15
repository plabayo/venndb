use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(key, any)]
    country: String,
}

fn main() {}
