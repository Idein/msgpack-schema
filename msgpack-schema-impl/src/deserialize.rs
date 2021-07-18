use crate::attr;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DataStruct, DeriveInput, Error, Fields, FieldsNamed, Result};

pub fn derive(node: &DeriveInput) -> Result<TokenStream> {
    let attrs = attr::get(&node.attrs)?;
    attrs.disallow_optional()?;
    attrs.disallow_tag()?;
    match &node.data {
        Data::Struct(strut) => match &strut.fields {
            Fields::Named(fields) => {
                if attrs.untagged.is_some() {
                    derive_untagged_struct(node, strut, fields)
                } else {
                    derive_struct(node, strut, fields)
                }
            }
            Fields::Unnamed(fields) => {
                attrs.disallow_untagged()?;
                let len = fields.unnamed.len();
                match len {
                    0 => {
                        return Err(Error::new_spanned(
                            node,
                            "empty tuple structs as deserialize are not supported",
                        ));
                    }
                    1 => derive_newtype_struct(node, strut, &fields.unnamed[0]),
                    _ => {
                        return Err(Error::new_spanned(
                            node,
                            "tuple structs as deserialize are not supported",
                        ))
                    }
                }
            }
            Fields::Unit => {
                attrs.disallow_untagged()?;
                return Err(Error::new_spanned(
                    node,
                    "unit structs as deserialize are not supported",
                ));
            }
        },
        Data::Enum(enu) => {
            if attrs.untagged.is_some() {
                derive_untagged_enum(node, enu)
            } else {
                derive_enum(node, enu)
            }
        }
        Data::Union(_) => Err(Error::new_spanned(
            node,
            "union as deserialize are not supported",
        )),
    }
}

fn derive_struct(
    node: &DeriveInput,
    _strut: &DataStruct,
    named_fields: &FieldsNamed,
) -> Result<TokenStream> {
    let ty = &node.ident;
    let (impl_generics, ty_generics, where_clause) = node.generics.split_for_impl();

    let fn_body = {
        let mut members = vec![];
        let mut tags = vec![];
        for field in &named_fields.named {
            let ident = field.ident.clone().unwrap();
            let attrs = attr::get(&field.attrs)?;
            attrs.disallow_untagged()?;
            attrs.require_tag(field)?;
            attr::check_tag_uniqueness(attrs.tag.as_ref().unwrap(), &mut tags)?;
            let tag = attrs.tag.unwrap().tag;
            let opt = attrs.optional.is_some();
            let ty = field.ty.clone();
            members.push((ident, tag, opt, ty))
        }

        let mut init = vec![];
        for (ident, _tag, opt, ty) in &members {
            let push = if *opt {
                quote! {
                    let mut #ident: #ty = None;
                }
            } else {
                quote! {
                    let mut #ident: ::std::option::Option<#ty> = None;
                }
            };
            init.push(push);
        }

        let mut filters = vec![];
        for (ident, tag, _opt, _ty) in &members {
            filters.push(quote! {
                #tag => {
                    if #ident.is_some() {
                        return Err(::msgpack_schema::InvalidInputError.into());
                    }
                    #ident = Some(__deserializer.deserialize()?);
                }
            });
        }

        let mut ctors = vec![];
        for (ident, _tag, opt, _ty) in &members {
            let push = if *opt {
                quote! {
                    #ident,
                }
            } else {
                quote! {
                    #ident: #ident.ok_or(::msgpack_schema::ValidationError)?,
                }
            };
            ctors.push(push);
        }

        quote! {
            let __len = match __deserializer.deserialize_token()? {
                ::msgpack_schema::Token::Map(len) => len,
                _ => return Err(::msgpack_schema::ValidationError.into()),
            };

            #( #init )*
            for _ in 0..__len {
                let __tag: u32 = __deserializer.deserialize()?;
                match __tag {
                    #( #filters )*
                    _ => {
                        let ::msgpack_schema::value::Any = __deserializer.deserialize()?;
                    }
                }
            }
            Ok(Self {
                #( #ctors )*
            })
        }
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics ::msgpack_schema::Deserialize for #ty #ty_generics #where_clause {
            fn deserialize(__deserializer: &mut ::msgpack_schema::Deserializer) -> ::std::result::Result<Self, ::msgpack_schema::DeserializeError> {
                #fn_body
            }
        }
    };

    Ok(gen)
}

fn derive_newtype_struct(
    node: &DeriveInput,
    _strut: &DataStruct,
    field: &syn::Field,
) -> Result<TokenStream> {
    let ty = &node.ident;
    let (impl_generics, ty_generics, where_clause) = node.generics.split_for_impl();

    let attrs = attr::get(&field.attrs)?;
    attrs.disallow_tag()?;
    attrs.disallow_optional()?;
    attrs.disallow_untagged()?;

    let fn_body = quote! {
        __deserializer.deserialize().map(Self)
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics ::msgpack_schema::Deserialize for #ty #ty_generics #where_clause {
            fn deserialize(__deserializer: &mut ::msgpack_schema::Deserializer) -> ::std::result::Result<Self, ::msgpack_schema::DeserializeError> {
                #fn_body
            }
        }
    };

    Ok(gen)
}

fn derive_enum(node: &DeriveInput, enu: &DataEnum) -> Result<TokenStream> {
    let ty = &node.ident;
    let (impl_generics, ty_generics, where_clause) = node.generics.split_for_impl();

    let fn_body = {
        let mut clauses = vec![];
        let mut tags = vec![];
        for variant in &enu.variants {
            let ident = variant.ident.clone();
            let attrs = attr::get(&variant.attrs)?;
            attrs.disallow_optional()?;
            attrs.disallow_untagged()?;
            attrs.require_tag(variant)?;
            attr::check_tag_uniqueness(attrs.tag.as_ref().unwrap(), &mut tags)?;
            let tag = attrs.tag.unwrap().tag;
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
                                #tag => {
                                    if __is_array {
                                        return Err(::msgpack_schema::ValidationError.into());
                                    }
                                    Ok(Self::#ident())
                                }
                            });
                        }
                        1 => {
                            let attrs = attr::get(&fields.unnamed[0].attrs)?;
                            attrs.disallow_optional()?;
                            attrs.disallow_tag()?;
                            attrs.disallow_untagged()?;
                            clauses.push(quote! {
                                #tag => {
                                    if !__is_array {
                                        return Err(::msgpack_schema::ValidationError.into());
                                    }
                                    Ok(Self::#ident(__deserializer.deserialize()?))
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
                        #tag => {
                            Ok(Self::#ident)
                        }
                    });
                }
            }
        }

        quote! {
            let (__tag, __is_array): (u32, bool) = match __deserializer.deserialize_token()? {
                ::msgpack_schema::Token::Int(v) => {
                    (<u32 as ::std::convert::TryFrom<_>>::try_from(v).map_err(|_| ::msgpack_schema::ValidationError)?, false)
                }
                ::msgpack_schema::Token::Array(len) => {
                    if len != 2 {
                        return Err(::msgpack_schema::ValidationError.into());
                    }
                    (__deserializer.deserialize::<u32>()?, true)
                }
                _ => {
                    return Err(::msgpack_schema::ValidationError.into());
                }
            };
            match __tag {
                #( #clauses )*
                _ => Err(::msgpack_schema::ValidationError.into()),
            }
        }
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics ::msgpack_schema::Deserialize for #ty #ty_generics #where_clause {
            fn deserialize(__deserializer: &mut ::msgpack_schema::Deserializer) -> ::std::result::Result<Self, ::msgpack_schema::DeserializeError> {
                #fn_body
            }
        }
    };

    Ok(gen)
}

fn derive_untagged_enum(node: &DeriveInput, enu: &DataEnum) -> Result<TokenStream> {
    let ty = &node.ident;
    let (impl_generics, ty_generics, where_clause) = node.generics.split_for_impl();

    let fn_body = {
        let mut members = vec![];
        for variant in &enu.variants {
            let attrs = attr::get(&variant.attrs)?;
            attrs.disallow_optional()?;
            attrs.disallow_tag()?;
            attrs.disallow_untagged()?;
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
                        let attrs = attr::get(&fields.unnamed[0].attrs)?;
                        attrs.disallow_optional()?;
                        attrs.disallow_tag()?;
                        attrs.disallow_untagged()?;
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
        for (variant, field) in &members {
            let ident = variant.ident.clone();
            let ty = field.ty.clone();
            clauses.push(quote! {
                if let Some(x) = __deserializer.try_deserialize::<#ty>()? {
                    return Ok(Self::#ident(x));
                }
            })
        }

        quote! {
            #( #clauses )*
            Err(::msgpack_schema::ValidationError.into())
        }
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics ::msgpack_schema::Deserialize for #ty #ty_generics #where_clause {
            fn deserialize(__deserializer: &mut ::msgpack_schema::Deserializer) -> ::std::result::Result<Self, ::msgpack_schema::DeserializeError> {
                #fn_body
            }
        }
    };

    Ok(gen)
}

fn derive_untagged_struct(
    node: &DeriveInput,
    _strut: &DataStruct,
    named_fields: &FieldsNamed,
) -> Result<TokenStream> {
    let ty = &node.ident;
    let (impl_generics, ty_generics, where_clause) = node.generics.split_for_impl();

    let fn_body = {
        let mut members = vec![];
        for field in &named_fields.named {
            let attrs = attr::get(&field.attrs)?;
            attrs.disallow_tag()?;
            attrs.disallow_optional()?;
            attrs.disallow_untagged()?;
            let ident = field.ident.clone().unwrap();
            let ty = field.ty.clone();
            members.push((ident, ty))
        }

        let len = members.len() as u32;

        let mut init = vec![];
        for (ident, ty) in &members {
            let push = quote! {
                let mut #ident: #ty = __deserializer.deserialize()?;
            };
            init.push(push);
        }

        let mut ctors = vec![];
        for (ident, _ty) in &members {
            let push = quote! {
                #ident,
            };
            ctors.push(push);
        }

        quote! {
            let __len = match __deserializer.deserialize_token()? {
                Token::Array(len) => len,
                _ => return Err(::msgpack_schema::ValidationError.into()),
            };

            if __len != #len {
                return Err(::msgpack_schema::ValidationError.into());
            }
            #( #init )*
            Ok(Self {
                #( #ctors )*
            })
        }
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics ::msgpack_schema::Deserialize for #ty #ty_generics #where_clause {
            fn deserialize(__deserializer: &mut ::msgpack_schema::Deserializer) -> ::std::result::Result<Self, ::msgpack_schema::DeserializeError> {
                #fn_body
            }
        }
    };

    Ok(gen)
}
