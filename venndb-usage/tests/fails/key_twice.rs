use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Employee {
    #[venndb(key = id, key)]
    id: u32,
}

fn id(employee: &Employee) -> u32 {
    employee.id
}

fn main() {}
