use msgpack_schema::*;

#[derive(Serialize)]
struct S1 {
    #[untagged]
    x: String,
}

#[derive(Serialize)]
struct S2(#[tag = 1] String);

#[derive(Serialize)]
struct S3(#[optional] String);

#[derive(Serialize)]
struct S4(#[untagged] String);

#[derive(Serialize)]
#[untagged]
struct S5 {
    #[tag = 0]
    x: String,
}

#[derive(Serialize)]
#[untagged]
struct S6 {
    #[untagged]
    x: String,
}

#[derive(Serialize)]
#[untagged]
struct S7 {
    #[optional]
    x: String,
}

#[derive(Deserialize)]
struct S8 {
    #[untagged]
    x: String,
}

#[derive(Deserialize)]
struct S9(#[tag = 1] String);

#[derive(Deserialize)]
struct S10(#[optional] String);

#[derive(Deserialize)]
struct S11(#[untagged] String);

#[derive(Deserialize)]
#[untagged]
struct S12 {
    #[tag = 0]
    x: String,
}

#[derive(Deserialize)]
#[untagged]
struct S13 {
    #[untagged]
    x: String,
}

#[derive(Deserialize)]
#[untagged]
struct S14 {
    #[optional]
    x: String,
}

fn main() {}
