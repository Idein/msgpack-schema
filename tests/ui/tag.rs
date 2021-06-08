use msgpack_schema::*;

#[derive(Serialize)]
struct S1 {
    #[tag = 0]
    age: u32,
    name: String,
}

#[derive(Deserialize)]
struct S2 {
    #[tag = 0]
    age: u32,
    name: String,
}

#[derive(Serialize)]
struct S3 {
    #[tag = 0]
    #[tag = 1]
    name: String,
}

#[derive(Deserialize)]
struct S4 {
    #[tag = 0]
    #[tag = 1]
    age: u32,
    name: String,
}

#[derive(Serialize)]
enum E1 {
    #[tag = 0]
    Cat,
    Dog,
}

#[derive(Deserialize)]
enum E2 {
    #[tag = 0]
    Cat,
    Dog,
}

#[derive(Serialize)]
enum E3 {
    #[tag = 0]
    #[tag = 1]
    Cat,
    Dog,
}

#[derive(Deserialize)]
enum E4 {
    #[tag = 0]
    #[tag = 1]
    Cat,
    Dog,
}

fn main() {}
