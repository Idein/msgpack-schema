use crate::{InvalidInputError, Token};

pub trait Deserializer: Clone {
    fn deserialize(&mut self) -> Result<Token, InvalidInputError>;
}
