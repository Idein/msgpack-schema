use crate::common;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    spanned::Spanned, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, FieldsNamed, Result,
    Visibility,
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
            "union as deserialize are not supported",
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
            return Err(Error::new_spanned(
                node,
                "unit structs as deserialize are not supported",
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
    let deserialize_trait = spanned_deserialize_trait(node);

    let fn_body = {
        let mut members = vec![];
        for field in &named_fields.named {
            let ident = field.ident.clone().unwrap();
            let tag = common::get_field_tag(&field)?;
            let opt = common::is_optional(&field)?;
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
                        return Err(::msgpack_schema::DeserializeError::DuplicatedField);
                    }
                    #ident = Some(::msgpack_schema::Deserialize::deserialize(__deserializer)?);
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
                    #ident: #ident.ok_or(::msgpack_schema::DeserializeError::MissingField)?,
                }
            };
            ctors.push(push);
        }

        quote! {
            let __len = __deserializer
                .deserialize()?
                .to_map()
                .ok_or(::msgpack_schema::DeserializeError::InvalidType)?;

            #( #init )*
            for _ in 0..__len {
                let __tag: u32 = ::msgpack_schema::Deserialize::deserialize(__deserializer)?;
                match __tag {
                    #( #filters )*
                    _ => {
                        let ::msgpack_schema::Any = ::msgpack_schema::Deserialize::deserialize(__deserializer)?;
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
        impl #impl_generics #deserialize_trait for #ty #ty_generics #where_clause {
            fn deserialize<D>(__deserializer: &mut D) -> ::std::result::Result<Self, ::msgpack_schema::DeserializeError<D::Error>>
            where
                D: ::msgpack_schema::Deserializer,
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
    _unnamed: &syn::Field,
) -> Result<TokenStream> {
    let ty = &node.ident;
    let (impl_generics, ty_generics, where_clause) = node.generics.split_for_impl();
    let deserialize_trait = spanned_deserialize_trait(node);

    let fn_body = quote! {
        ::msgpack_schema::Deserialize::deserialize(__deserializer).map(Self)
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #deserialize_trait for #ty #ty_generics #where_clause {
            fn deserialize<D>(__deserializer: &mut D) -> ::std::result::Result<Self, ::msgpack_schema::DeserializeError<D::Error>>
            where
                D: ::msgpack_schema::Deserializer,
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
    let deserialize_trait = spanned_deserialize_trait(node);

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
                                #tag => {
                                    if __is_array {
                                        return Err(::msgpack_schema::DeserializeError::InvalidType);
                                    }
                                    Ok(Self::#ident())
                                }
                            });
                        }
                        1 => {
                            clauses.push(quote! {
                                #tag => {
                                    if !__is_array {
                                        return Err(::msgpack_schema::DeserializeError::InvalidType);
                                    }
                                    Ok(Self::#ident(Deserialize::deserialize(__deserializer)?))
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
            let (__tag, __is_array): (u32, bool) = match __deserializer.deserialize()? {
                ::msgpack_schema::Token::Int(v) => {
                    (<u32 as ::std::convert::TryFrom<_>>::try_from(v).map_err(|_| ::msgpack_schema::DeserializeError::IntegerOutOfRange)?, false)
                }
                ::msgpack_schema::Token::Array(len) => {
                    if len != 2 {
                        return Err(::msgpack_schema::DeserializeError::InvalidLength);
                    }
                    (u32::deserialize(__deserializer)?, true)
                }
                _ => {
                    return Err(::msgpack_schema::DeserializeError::InvalidType);
                }
            };
            match __tag {
                #( #clauses )*
                _ => Err(::msgpack_schema::DeserializeError::UnknownVariant),
            }
        }
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #deserialize_trait for #ty #ty_generics #where_clause {
            fn deserialize<D>(__deserializer: &mut D) -> ::std::result::Result<Self, ::msgpack_schema::DeserializeError<D::Error>>
            where
                D: ::msgpack_schema::Deserializer,
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
    let deserialize_trait = spanned_deserialize_trait(node);

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
        for (variant, field) in &members {
            let ident = variant.ident.clone();
            let ty = field.ty.clone();
            clauses.push(quote! {
                if let Ok(x) = ::msgpack_schema::value::deserialize::<#ty>(__value.clone()) {
                    return Ok(Self::#ident(x));
                }
            })
        }

        quote! {
            let __value = ::msgpack_schema::Value::deserialize(__deserializer)?;
            #( #clauses )*
            Err(::msgpack_schema::DeserializeError::UnknownVariant)
        }
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #deserialize_trait for #ty #ty_generics #where_clause {
            fn deserialize<D>(__deserializer: &mut D) -> ::std::result::Result<Self, ::msgpack_schema::DeserializeError<D::Error>>
            where
                D: ::msgpack_schema::Deserializer,
            {
                #fn_body
            }
        }
    };

    Ok(gen)
}

fn spanned_deserialize_trait(input: &DeriveInput) -> TokenStream {
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
        quote_spanned!(span => Deserialize)
    };
    quote!(#path #serialize)
}
