use msgpack_schema::*;

mod serialize {
    use super::*;

    #[derive(Serialize)]
    struct S1 {
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

    #[derive(Serialize)]
    #[untagged]
    #[untagged]
    struct S5 {}
}

mod deserialize {
    use super::*;

    #[derive(Deserialize)]
    struct S1 {
        #[optional]
        #[optional]
        #[tag = 1]
        x: Option<String>,
    }

    #[derive(Deserialize)]
    struct S2 {
        #[tag = 1]
        #[tag = 2]
        x: String,
    }

    #[derive(Deserialize)]
    #[untagged]
    #[untagged]
    struct S3 {}
}

fn main() {}
