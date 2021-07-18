use msgpack_schema::*;

mod serialize {
    use super::*;

    #[derive(Serialize)]
    enum E1 {
        #[optional]
        V1,
    }

    #[derive(Serialize)]
    enum E2 {
        #[untagged]
        V1,
    }

    #[derive(Serialize)]
    enum E3 {
        #[tag = 1]
        V1(#[untagged] String),
    }

    #[derive(Serialize)]
    enum E4 {
        #[tag = 1]
        V1(#[optional] String),
    }

    #[derive(Serialize)]
    enum E5 {
        #[tag = 1]
        V1(#[tag = 1] String),
    }

    #[derive(Serialize)]
    #[untagged]
    enum E6 {
        #[optional]
        V1,
    }

    #[derive(Serialize)]
    #[untagged]
    enum E7 {
        #[untagged]
        V1,
    }

    #[derive(Serialize)]
    #[untagged]
    enum E8 {
        #[tag = 1]
        V1,
    }

    #[derive(Serialize)]
    #[untagged]
    enum E9 {
        V1(#[untagged] String),
    }

    #[derive(Serialize)]
    #[untagged]
    enum E10 {
        V1(#[optional] String),
    }

    #[derive(Serialize)]
    #[untagged]
    enum E11 {
        V1(#[tag = 1] String),
    }
}

mod deserialize {
    use super::*;

    #[derive(Deserialize)]
    enum E1 {
        #[optional]
        V1,
    }

    #[derive(Deserialize)]
    enum E2 {
        #[untagged]
        V1,
    }

    #[derive(Deserialize)]
    enum E3 {
        #[tag = 1]
        V1(#[untagged] String),
    }

    #[derive(Deserialize)]
    enum E4 {
        #[tag = 1]
        V1(#[optional] String),
    }

    #[derive(Deserialize)]
    enum E5 {
        #[tag = 1]
        V1(#[tag = 1] String),
    }

    #[derive(Deserialize)]
    #[untagged]
    enum E6 {
        #[optional]
        V1,
    }

    #[derive(Deserialize)]
    #[untagged]
    enum E7 {
        #[untagged]
        V1,
    }

    #[derive(Deserialize)]
    #[untagged]
    enum E8 {
        #[tag = 1]
        V1,
    }

    #[derive(Deserialize)]
    #[untagged]
    enum E9 {
        V1(#[untagged] String),
    }

    #[derive(Deserialize)]
    #[untagged]
    enum E10 {
        V1(#[optional] String),
    }

    #[derive(Deserialize)]
    #[untagged]
    enum E11 {
        V1(#[tag = 1] String),
    }
}

fn main() {}
