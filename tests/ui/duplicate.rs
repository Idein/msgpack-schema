use msgpack_schema::*;

#[derive(Serialize)]
struct S1 {
    #[optional]
    #[optional]
    #[tag = 1]
    x: Option<String>,
}

#[derive(Deserialize)]
struct S2 {
    #[optional]
    #[optional]
    #[tag = 1]
    x: Option<String>,
}

#[derive(Serialize)]
struct S3 {
    #[tag = 1]
    #[tag = 2]
    x: String,
}

#[derive(Deserialize)]
struct S4 {
    #[tag = 1]
    #[tag = 2]
    x: String,
}

#[derive(Serialize)]
#[untagged]
#[untagged]
struct S5 {}

#[derive(Deserialize)]
#[untagged]
#[untagged]
struct S6 {}

fn main() {}
