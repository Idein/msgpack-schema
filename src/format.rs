use crate::{value::Int, InvalidInputError, Token};

pub trait Serializer {
    fn serialize_nil(&mut self);
    fn serialize_bool(&mut self, v: bool);
    fn serialize_int(&mut self, v: Int);
    fn serialize_f32(&mut self, v: f32);
    fn serialize_f64(&mut self, v: f64);
    fn serialize_str(&mut self, v: &[u8]);
    fn serialize_bin(&mut self, v: &[u8]);
    fn serialize_array(&mut self, len: u32);
    fn serialize_map(&mut self, len: u32);
    fn serialize_ext(&mut self, tag: i8, v: &[u8]);
}

pub trait Deserializer: Clone {
    fn deserialize(&mut self) -> Result<Token, InvalidInputError>;
}
