use msgpack_schema::*;

#[derive(Serialize)]
struct S1 {
    #[tag = 0]
    x: String,
    #[tag = 0]
    y: String,
}

#[derive(Deserialize)]
struct S2 {
    #[tag = 0]
    x: String,
    #[tag = 0]
    y: String,
}

#[derive(Serialize)]
enum E1 {
    #[tag = 0]
    V1,
    #[tag = 0]
    V2,
}

#[derive(Deserialize)]
enum E2 {
    #[tag = 0]
    V1,
    #[tag = 0]
    V2,
}

fn main() {}
