mod attr;
mod deserialize;
mod serialize;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// The `schema` attribute is experimental.
#[proc_macro_derive(Serialize, attributes(schema, tag, optional, untagged, flatten))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    serialize::derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// The `schema` attribute is experimental.
#[proc_macro_derive(Deserialize, attributes(schema, tag, optional, untagged, flatten))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    deserialize::derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
