use msgpack_schema::*;

mod serialize {
    use super::*;

    #[derive(Serialize)]
    struct S1;

    #[derive(Serialize)]
    struct S3();
}

mod deserialize {
    use super::*;

    #[derive(Deserialize)]
    struct S2;

    #[derive(Deserialize)]
    struct S4();
}

fn main() {}
