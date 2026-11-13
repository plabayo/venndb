use venndb::Any;

#[derive(Debug, Any)]
enum Department {
    #[any(always)]
    Any,
    Hr,
}

fn main() {}
