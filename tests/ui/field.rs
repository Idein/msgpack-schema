use msgpack_schema::*;

mod serialize {
    use super::*;

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
    struct S8(#[flatten] String);

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

    #[derive(Serialize)]
    #[untagged]
    struct S9 {
        #[flatten]
        x: String,
    }

    #[derive(Serialize)]
    struct S10 {
        #[tag = 1]
        #[flatten]
        x: String,
    }

    #[derive(Serialize)]
    struct S11 {
        #[optional]
        #[flatten]
        x: String,
    }
}

mod deserialize {
    use super::*;

    #[derive(Deserialize)]
    struct S1 {
        #[untagged]
        x: String,
    }

    #[derive(Deserialize)]
    struct S2(#[tag = 1] String);

    #[derive(Deserialize)]
    struct S3(#[optional] String);

    #[derive(Deserialize)]
    struct S4(#[untagged] String);

    #[derive(Deserialize)]
    struct S8(#[flatten] String);

    #[derive(Deserialize)]
    #[untagged]
    struct S5 {
        #[tag = 0]
        x: String,
    }

    #[derive(Deserialize)]
    #[untagged]
    struct S6 {
        #[untagged]
        x: String,
    }

    #[derive(Deserialize)]
    #[untagged]
    struct S7 {
        #[optional]
        x: String,
    }

    #[derive(Deserialize)]
    #[untagged]
    struct S9 {
        #[flatten]
        x: String,
    }

    #[derive(Deserialize)]
    struct S10 {
        #[tag = 1]
        #[flatten]
        x: String,
    }

    #[derive(Deserialize)]
    struct S11 {
        #[optional]
        #[flatten]
        x: String,
    }
}

fn main() {}