mod common;
mod deserialize;
mod serialize;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Serialize, attributes(tag, optional, untagged))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    serialize::derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(Deserialize, attributes(tag, optional, untagged))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    deserialize::derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
