use crate::common;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    spanned::Spanned, Data, DataEnum, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed,
    Result, Visibility,
};

pub fn derive(node: &DeriveInput) -> Result<TokenStream> {
    match &node.data {
        Data::Struct(strut) => derive_struct(node, strut),
        Data::Enum(enu) => {
            if common::has_untagged(&node.attrs) {
                derive_untagged_enum(node, enu)
            } else {
                derive_enum(node, enu)
            }
        }
        Data::Union(_) => Err(Error::new_spanned(
            node,
            "union as serialize are not supported",
        )),
    }
}

fn derive_struct(node: &DeriveInput, strut: &DataStruct) -> Result<TokenStream> {
    match &strut.fields {
        Fields::Named(fields) => derive_c_struct(node, strut, fields),
        Fields::Unnamed(fields) => {
            let len = fields.unnamed.len();
            match len {
                0 => {
                    return Err(Error::new_spanned(
                        node,
                        "empty tuple structs as serialize are not supported",
                    ));
                }
                1 => derive_newtype_struct(node, strut, &fields.unnamed[0]),
                _ => {
                    return Err(Error::new_spanned(
                        node,
                        "tuple structs as serialize are not supported",
                    ))
                }
            }
        }
        Fields::Unit => {
            return Err(Error::new_spanned(
                node,
                "unit structs as serialize are not supported",
            ));
        }
    }
}

fn derive_c_struct(
    node: &DeriveInput,
    _strut: &DataStruct,
    named_fields: &FieldsNamed,
) -> Result<TokenStream> {
    let ty = &node.ident;
    let (impl_generics, ty_generics, where_clause) = node.generics.split_for_impl();
    let serialize_trait = spanned_serialize_trait(node);

    let fn_body = {
        let mut members = vec![];
        for field in &named_fields.named {
            let ident = field.ident.clone().unwrap();
            let tag = common::get_field_tag(&field)?;
            // TODO: require `#[required]` or `#[optional]` for fields of the Option<T> type
            let opt = common::is_optional(&field)?;
            members.push((ident, tag, opt));
        }

        let max_len = named_fields.named.len() as u32;

        let mut decs = vec![];
        for (ident, _tag, opt) in &members {
            if *opt {
                decs.push(quote! {
                    if self.#ident.is_none() {
                        max_len -= 1;
                    }
                })
            };
        }

        let mut pushes = vec![];
        for (ident, tag, opt) in &members {
            let push = if *opt {
                quote! {
                    if let Some(value) = &self.#ident {
                        (#tag as u32).serialize(serializer);
                        value.serialize(serializer);
                    }
                }
            } else {
                quote! {
                    (#tag as u32).serialize(serializer);
                    self.#ident.serialize(serializer);
                }
            };
            pushes.push(push);
        }

        quote! {
            let mut max_len: u32 = #max_len;
            #( #decs )*
            serializer.serialize_map(max_len);
            #( #pushes )*
        }
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #serialize_trait for #ty #ty_generics #where_clause {
            fn serialize<S>(&self, serializer: &mut S)
            where
                S: ::msgpack_schema::Serializer
            {
                #fn_body
            }
        }
    };

    Ok(gen)
}

fn derive_newtype_struct(
    node: &DeriveInput,
    _strut: &DataStruct,
    _field: &Field,
) -> Result<TokenStream> {
    let ty = &node.ident;
    let (impl_generics, ty_generics, where_clause) = node.generics.split_for_impl();
    let serialize_trait = spanned_serialize_trait(node);

    let fn_body = quote! {
        self.0.serialize(serializer)
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #serialize_trait for #ty #ty_generics #where_clause {
            fn serialize<S>(&self, serializer: &mut S)
            where
                S: ::msgpack_schema::Serializer
            {
                #fn_body
            }
        }
    };

    Ok(gen)
}

fn derive_enum(node: &DeriveInput, enu: &DataEnum) -> Result<TokenStream> {
    let ty = &node.ident;
    let (impl_generics, ty_generics, where_clause) = node.generics.split_for_impl();
    let serialize_trait = spanned_serialize_trait(node);

    let fn_body = {
        let mut clauses = vec![];
        for variant in &enu.variants {
            let ident = variant.ident.clone();
            let tag = common::get_variant_tag(&variant)?;
            match &variant.fields {
                Fields::Named(_) => {
                    return Err(Error::new_spanned(
                        node,
                        "variants with fields are not supported",
                    ));
                }
                Fields::Unnamed(fields) => {
                    let len = fields.unnamed.len() as u32;
                    match len {
                        0 => {
                            clauses.push(quote! {
                                Self::#ident() => {
                                    (#tag as u32).serialize(serializer)
                                }
                            });
                        }
                        1 => {
                            clauses.push(quote! {
                                Self::#ident(value) => {
                                    serializer.serialize_array(2);
                                    (#tag as u32).serialize(serializer);
                                    value.serialize(serializer)
                                }
                            });
                        }
                        _ => {
                            return Err(Error::new_spanned(
                                node,
                                "tuple variants with more than one elements are not supported",
                            ));
                        }
                    }
                }
                Fields::Unit => {
                    clauses.push(quote! {
                        Self::#ident => {
                            (#tag as u32).serialize(serializer)
                        }
                    });
                }
            }
        }

        quote! {
            match self {
                #( #clauses )*
            }
        }
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #serialize_trait for #ty #ty_generics #where_clause {
            fn serialize<S>(&self, serializer: &mut S)
            where
                S: ::msgpack_schema::Serializer
            {
                #fn_body
            }
        }
    };

    Ok(gen)
}

fn derive_untagged_enum(node: &DeriveInput, enu: &DataEnum) -> Result<TokenStream> {
    let ty = &node.ident;
    let (impl_generics, ty_generics, where_clause) = node.generics.split_for_impl();
    let serialize_trait = spanned_serialize_trait(node);

    let fn_body = {
        let mut members = vec![];
        for variant in &enu.variants {
            match &variant.fields {
                Fields::Named(_) => {
                    return Err(Error::new_spanned(
                        node,
                        "struct variants cannot be untagged",
                    ));
                }
                Fields::Unnamed(fields) => match fields.unnamed.len() {
                    0 => {
                        return Err(Error::new_spanned(
                            node,
                            "empty tuple variants cannot be untagged",
                        ));
                    }
                    1 => {
                        members.push((variant, &fields.unnamed[0]));
                    }
                    _ => {
                        return Err(Error::new_spanned(
                            node,
                            "tuple variants cannot be untagged",
                        ));
                    }
                },
                Fields::Unit => {
                    return Err(Error::new_spanned(
                        node,
                        "unit variants cannot be supported",
                    ));
                }
            }
        }

        let mut clauses = vec![];
        for (variant, _field) in &members {
            let ident = variant.ident.clone();
            clauses.push(quote! {
                Self::#ident(value) => {
                    value.serialize(serializer)
                }
            });
        }

        quote! {
            match self {
                #( #clauses )*
            }
        }
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #serialize_trait for #ty #ty_generics #where_clause {
            fn serialize<S>(&self, serializer: &mut S)
            where
                S: ::msgpack_schema::Serializer
            {
                #fn_body
            }
        }
    };

    Ok(gen)
}

fn spanned_serialize_trait(input: &DeriveInput) -> TokenStream {
    let path = {
        let span = match &input.vis {
            Visibility::Public(vis) => vis.pub_token.span(),
            Visibility::Crate(vis) => vis.crate_token.span(),
            Visibility::Restricted(vis) => vis.pub_token.span(),
            Visibility::Inherited => match &input.data {
                Data::Struct(data) => data.struct_token.span(),
                Data::Enum(data) => data.enum_token.span(),
                Data::Union(data) => data.union_token.span(),
            },
        };
        quote_spanned!(span => ::msgpack_schema::)
    };
    let serialize = {
        let span = input.ident.span();
        quote_spanned!(span => Serialize)
    };
    quote!(#path #serialize)
}
