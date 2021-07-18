use msgpack_schema::*;

#[derive(Serialize)]
#[tag = 1]
struct S1 {
    x: String,
}

#[derive(Serialize)]
#[optional]
struct S2 {
    x: String,
}

#[derive(Serialize)]
#[untagged]
struct S3(String);

#[derive(Serialize)]
#[untagged]
struct S4;

#[derive(Deserialize)]
#[tag = 1]
struct S5 {
    x: String,
}

#[derive(Deserialize)]
#[optional]
struct S6 {
    x: String,
}

#[derive(Deserialize)]
#[untagged]
struct S7(String);

#[derive(Deserialize)]
#[untagged]
struct S8;

fn main() {}
