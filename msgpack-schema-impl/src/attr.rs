use quote::ToTokens;
use syn::{
    parse::{Nothing, ParseStream, Parser},
    Attribute, Error, LitInt, Result, Token,
};

pub struct Attrs<'a> {
    pub tag: Option<Tag<'a>>,
    pub optional: Option<&'a Attribute>,
    pub untagged: Option<&'a Attribute>,
}

#[derive(Clone)]
pub struct Tag<'a> {
    pub original: &'a Attribute,
    pub tag: LitInt,
}

pub fn get(attrs: &[Attribute]) -> Result<Attrs> {
    let mut output = Attrs {
        tag: None,
        optional: None,
        untagged: None,
    };

    for attr in attrs {
        if attr.path.is_ident("tag") {
            let parser = |input: ParseStream| {
                let _eq_token: Token![=] = input.parse()?;
                let lit_int: LitInt = input.parse()?;
                Ok(lit_int)
            };
            let tag = parser.parse2(attr.tokens.clone())?;
            if output.tag.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[tag] attribute"));
            }
            output.tag = Some(Tag {
                original: attr,
                tag,
            })
        } else if attr.path.is_ident("untagged") {
            require_empty_attribute(attr)?;
            if output.untagged.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[untagged] attribute"));
            }
            output.untagged = Some(attr);
        } else if attr.path.is_ident("optional") {
            require_empty_attribute(attr)?;
            if output.optional.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[optional] attribute"));
            }
            output.optional = Some(attr);
        }
    }
    Ok(output)
}

fn require_empty_attribute(attr: &Attribute) -> Result<()> {
    syn::parse2::<Nothing>(attr.tokens.clone())?;
    Ok(())
}

impl<'a> Attrs<'a> {
    pub fn disallow_tag(&self) -> Result<()> {
        if let Some(tag) = self.tag.clone() {
            return Err(Error::new_spanned(
                tag.original,
                "#[tag] at an invalid position",
            ));
        }
        Ok(())
    }

    pub fn disallow_optional(&self) -> Result<()> {
        if let Some(original) = self.optional {
            return Err(Error::new_spanned(
                original,
                "#[optional] at an invalid position",
            ));
        }
        Ok(())
    }

    pub fn disallow_untagged(&self) -> Result<()> {
        if let Some(original) = self.untagged {
            return Err(Error::new_spanned(
                original,
                "#[untagged] at an invalid position",
            ));
        }
        Ok(())
    }

    pub fn require_tag(&self, tokens: impl ToTokens) -> Result<()> {
        if self.tag.is_none() {
            return Err(Error::new_spanned(tokens, "no #[tag] given"));
        }
        Ok(())
    }
}
