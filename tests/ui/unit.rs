use msgpack_schema::*;

#[derive(Serialize)]
struct S1;

#[derive(Deserialize)]
struct S2;

#[derive(Serialize)]
struct S3();

#[derive(Deserialize)]
struct S4();

fn main() {}
