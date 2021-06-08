use syn::{
    parse::{ParseStream, Parser},
    Attribute, Error, Field, LitInt, Result, Token, Variant,
};

pub fn has_untagged(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .find(|attr| attr.path.is_ident("untagged"))
        .is_some()
}

pub fn get_field_tag(field: &Field) -> Result<LitInt> {
    let attrs: Vec<_> = field
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("tag"))
        .collect();
    if attrs.is_empty() {
        return Err(Error::new_spanned(field, "no #[tag] given"));
    }
    if attrs.len() > 1 {
        return Err(Error::new_spanned(field, "more than one #[tag] given"));
    }
    let attr = &attrs[0];
    let parser = |input: ParseStream| {
        let _eq_token: Token![=] = input.parse()?;
        let lit_int: LitInt = input.parse()?;
        Ok(lit_int)
    };
    parser.parse2(attr.tokens.clone())
}

pub fn get_variant_tag(variant: &Variant) -> Result<LitInt> {
    let attrs: Vec<_> = variant
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("tag"))
        .collect();
    if attrs.is_empty() {
        return Err(Error::new_spanned(variant, "no #[tag] given"));
    }
    if attrs.len() > 1 {
        return Err(Error::new_spanned(variant, "more than one #[tag] given"));
    }
    let attr = &attrs[0];
    let parser = |input: ParseStream| {
        let _eq_token: Token![=] = input.parse()?;
        let lit_int: LitInt = input.parse()?;
        Ok(lit_int)
    };
    parser.parse2(attr.tokens.clone())
}

pub fn is_optional(field: &Field) -> Result<bool> {
    let attrs: Vec<_> = field
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("optional"))
        .collect();
    if attrs.is_empty() {
        return Ok(false);
    }
    if attrs.len() > 1 {
        return Err(Error::new_spanned(field, "more than one #[optional] given"));
    }
    let attr = &attrs[0];
    if !attr.tokens.is_empty() {
        return Err(Error::new_spanned(field, "#[optional] takes no arguments"));
    }
    Ok(true)
}
