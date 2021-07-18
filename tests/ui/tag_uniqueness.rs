use msgpack_schema::*;

mod serialize {
    use super::*;

    #[derive(Serialize)]
    struct S1 {
        #[tag = 0]
        x: String,
        #[tag = 0]
        y: String,
    }

    #[derive(Serialize)]
    enum E1 {
        #[tag = 0]
        V1,
        #[tag = 0]
        V2,
    }
}

mod deserialize {
    use super::*;

    #[derive(Deserialize)]
    struct S1 {
        #[tag = 0]
        x: String,
        #[tag = 0]
        y: String,
    }

    #[derive(Deserialize)]
    enum E1 {
        #[tag = 0]
        V1,
        #[tag = 0]
        V2,
    }
}

fn main() {}
