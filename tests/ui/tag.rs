use msgpack_schema::*;

mod serialize {
    use super::*;

    #[derive(Serialize)]
    struct S1 {
        x: String,
    }

    #[derive(Serialize)]
    enum E1 {
        V,
    }
}

mod deserialize {
    use super::*;

    #[derive(Deserialize)]
    struct S1 {
        x: String,
    }

    #[derive(Deserialize)]
    enum E1 {
        V,
    }
}

fn main() {}
