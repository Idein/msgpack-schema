use std::str::FromStr;

use crate::attr;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed, FieldsUnnamed,
    Result,
};

pub fn derive(node: &DeriveInput) -> Result<TokenStream> {
    let attrs = attr::get(&node.attrs)?;
    attrs.disallow_optional()?;
    attrs.disallow_tag()?;
    attrs.disallow_flatten()?;
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
                    0 => Err(Error::new_spanned(
                        node,
                        "empty tuple structs as serialize are not supported",
                    )),
                    1 => derive_newtype_struct(node, strut, &fields.unnamed[0]),
                    _ => derive_tuple_struct(node, strut, fields),
                }
            }
            Fields::Unit => {
                attrs.disallow_untagged()?;
                Err(Error::new_spanned(
                    node,
                    "unit structs as serialize are not supported",
                ))
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
            "union as serialize are not supported",
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

    enum FieldKind {
        Ordinary(u32),
        Optional(u32),
        Flatten,
    }

    let fields = {
        let mut fields = vec![];
        let mut tags = vec![];
        for field in &named_fields.named {
            let ident = field.ident.clone().unwrap();
            let ty = field.ty.clone();
            let attrs = attr::get(&field.attrs)?;
            attrs.disallow_untagged()?;
            let kind = if attrs.flatten.is_some() {
                attrs.disallow_tag()?;
                attrs.disallow_optional()?;
                FieldKind::Flatten
            } else {
                attrs.require_tag(field)?;
                attr::check_tag_uniqueness(attrs.tag.as_ref().unwrap(), &mut tags)?;
                let tag = attrs.tag.unwrap().tag;
                // TODO: require `#[required]` or `#[optional]` for fields of the Option<T> type
                if attrs.optional.is_some() {
                    FieldKind::Optional(tag)
                } else {
                    FieldKind::Ordinary(tag)
                }
            };
            fields.push((ident, ty, kind));
        }
        fields
    };

    let count_fields_body = {
        let max_len = named_fields.named.len() as u32;

        let mut decs = vec![];
        for (ident, ty, kind) in &fields {
            match kind {
                FieldKind::Flatten => {
                    decs.push(quote! {
                        __max_len -= 1;
                        __max_len += <#ty as ::msgpack_schema::StructSerialize>::count_fields(&self.#ident);
                    });
                }
                FieldKind::Optional(_) => {
                    decs.push(quote! {
                        if self.#ident.is_none() {
                            __max_len -= 1;
                        }
                    });
                }
                FieldKind::Ordinary(_) => {}
            }
        }

        quote! {
            let mut __max_len: u32 = #max_len;
            #( #decs )*
            __max_len
        }
    };

    let serialize_fields_body = {
        let mut pushes = vec![];
        for (ident, ty, kind) in &fields {
            let code = match kind {
                FieldKind::Ordinary(tag) => {
                    quote! {
                        __serializer.serialize(#tag);
                        __serializer.serialize(&self.#ident);
                    }
                }
                FieldKind::Optional(tag) => {
                    quote! {
                        if let Some(__value) = &self.#ident {
                            __serializer.serialize(#tag);
                            __serializer.serialize(__value);
                        }
                    }
                }
                FieldKind::Flatten => {
                    quote! {
                        <#ty as ::msgpack_schema::StructSerialize>::serialize_fields(&self.#ident, __serializer);
                    }
                }
            };
            pushes.push(code);
        }

        quote! {
            #( #pushes )*
        }
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics ::msgpack_schema::Serialize for #ty #ty_generics #where_clause {
            fn serialize(&self, __serializer: &mut ::msgpack_schema::Serializer) {
                let count = <Self as ::msgpack_schema::StructSerialize>::count_fields(self);
                __serializer.serialize_map(count);
                <Self as ::msgpack_schema::StructSerialize>::serialize_fields(self, __serializer);
            }
        }

        #[allow(unused_qualifications)]
        impl #impl_generics ::msgpack_schema::StructSerialize for #ty #ty_generics #where_clause {
            fn count_fields(&self) -> u32 {
                #count_fields_body
            }

            fn serialize_fields(&self, __serializer: &mut ::msgpack_schema::Serializer) {
                #serialize_fields_body
            }
        }
    };

    Ok(gen)
}

fn derive_newtype_struct(
    node: &DeriveInput,
    _strut: &DataStruct,
    field: &Field,
) -> Result<TokenStream> {
    let ty = &node.ident;
    let (impl_generics, ty_generics, where_clause) = node.generics.split_for_impl();

    let attrs = attr::get(&field.attrs)?;
    attrs.disallow_tag()?;
    attrs.disallow_optional()?;
    attrs.disallow_untagged()?;
    attrs.disallow_flatten()?;

    let fn_body = quote! {
        __serializer.serialize(&self.0);
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics ::msgpack_schema::Serialize for #ty #ty_generics #where_clause {
            fn serialize(&self, __serializer: &mut ::msgpack_schema::Serializer) {
                #fn_body
            }
        }
    };

    Ok(gen)
}

fn derive_tuple_struct(
    node: &DeriveInput,
    _strut: &DataStruct,
    fields: &FieldsUnnamed,
) -> Result<TokenStream> {
    let ty = &node.ident;
    let (impl_generics, ty_generics, where_clause) = node.generics.split_for_impl();

    for field in &fields.unnamed {
        let attrs = attr::get(&field.attrs)?;
        attrs.disallow_tag()?;
        attrs.disallow_optional()?;
        attrs.disallow_untagged()?;
        attrs.disallow_flatten()?;
    }

    let count = fields.unnamed.len() as u32;
    let field_specs = (0..count).map(|n| TokenStream::from_str(&format!("{}", n)).unwrap());

    let fn_body = quote! {
        __serializer.serialize_array(#count);
        #( __serializer.serialize(&self.#field_specs); )*
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics ::msgpack_schema::Serialize for #ty #ty_generics #where_clause {
            fn serialize(&self, __serializer: &mut ::msgpack_schema::Serializer) {
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
            attrs.disallow_flatten()?;
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
                                Self::#ident() => {
                                    __serializer.serialize(#tag);
                                }
                            });
                        }
                        1 => {
                            let attrs = attr::get(&fields.unnamed[0].attrs)?;
                            attrs.disallow_optional()?;
                            attrs.disallow_tag()?;
                            attrs.disallow_untagged()?;
                            attrs.disallow_flatten()?;
                            clauses.push(quote! {
                                Self::#ident(__value) => {
                                    __serializer.serialize_array(2);
                                    __serializer.serialize(#tag);
                                    __serializer.serialize(__value);
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
                            __serializer.serialize(#tag);
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
        impl #impl_generics ::msgpack_schema::Serialize for #ty #ty_generics #where_clause {
            fn serialize(&self, __serializer: &mut ::msgpack_schema::Serializer) {
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
            attrs.disallow_flatten()?;
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
                        attrs.disallow_flatten()?;
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
                Self::#ident(__value) => {
                    __serializer.serialize(__value)
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
        impl #impl_generics ::msgpack_schema::Serialize for #ty #ty_generics #where_clause {
            fn serialize(&self, __serializer: &mut ::msgpack_schema::Serializer) {
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
            let ident = field.ident.clone().unwrap();
            let attrs = attr::get(&field.attrs)?;
            attrs.disallow_tag()?;
            attrs.disallow_optional()?;
            attrs.disallow_untagged()?;
            attrs.disallow_flatten()?;
            members.push(ident);
        }

        let len = members.len() as u32;

        let mut pushes = vec![];
        for ident in &members {
            let push = quote! {
                __serializer.serialize(&self.#ident);
            };
            pushes.push(push);
        }

        quote! {
            __serializer.serialize_array(#len);
            #( #pushes )*
        }
    };

    let gen = quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics ::msgpack_schema::Serialize for #ty #ty_generics #where_clause {
            fn serialize(&self, __serializer: &mut ::msgpack_schema::Serializer) {
                #fn_body
            }
        }
    };

    Ok(gen)
}
