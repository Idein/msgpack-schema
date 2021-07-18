use msgpack_schema::*;

mod serialize {
    use super::*;

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
    #[flatten]
    struct S5 {
        x: String,
    }

    #[derive(Serialize)]
    #[untagged]
    struct S3(String);

    #[derive(Serialize)]
    #[untagged]
    struct S4;
}

mod deserialize {
    use super::*;

    #[derive(Deserialize)]
    #[tag = 1]
    struct S1 {
        x: String,
    }

    #[derive(Deserialize)]
    #[optional]
    struct S2 {
        x: String,
    }

    #[derive(Deserialize)]
    #[flatten]
    struct S5 {
        x: String,
    }

    #[derive(Deserialize)]
    #[untagged]
    struct S3(String);

    #[derive(Deserialize)]
    #[untagged]
    struct S4;
}

fn main() {}
