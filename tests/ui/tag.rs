use msgpack_schema::*;

#[derive(Serialize)]
struct S1 {
    x: String,
}

#[derive(Deserialize)]
struct S2 {
    x: String,
}

#[derive(Serialize)]
enum E1 {
    V,
}

#[derive(Deserialize)]
enum E2 {
    V,
}

fn main() {}
