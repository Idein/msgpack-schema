use quote::ToTokens;
use syn::{
    parse::{Nothing, ParseStream, Parser},
    Attribute, Error, LitInt, Result, Token,
};

pub struct Attrs<'a> {
    pub tag: Option<Tag<'a>>,
    pub optional: Option<Optional<'a>>,
    pub untagged: Option<Untagged<'a>>,
}

#[derive(Clone)]
pub struct Tag<'a> {
    pub original: &'a Attribute,
    pub tag: LitInt,
}

#[derive(Clone)]
pub struct Optional<'a> {
    pub original: &'a Attribute,
}

#[derive(Clone)]
pub struct Untagged<'a> {
    pub original: &'a Attribute,
}

pub fn get(attrs: &[Attribute]) -> Result<Attrs> {
    let mut output = Attrs {
        tag: None,
        optional: None,
        untagged: None,
    };

    for attr in attrs {
        if attr.path.is_ident("schema") {
            parse_schema_attribute(&mut output, attr)?;
        } else if attr.path.is_ident("tag") {
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
            output.untagged = Some(Untagged { original: attr });
        } else if attr.path.is_ident("optional") {
            require_empty_attribute(attr)?;
            if output.optional.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[optional] attribute"));
            }
            output.optional = Some(Optional { original: attr });
        }
    }
    Ok(output)
}

fn parse_schema_attribute<'a>(output: &mut Attrs<'a>, attr: &'a Attribute) -> Result<()> {
    syn::custom_keyword!(optional);
    syn::custom_keyword!(tag);
    syn::custom_keyword!(untagged);

    attr.parse_args_with(|input: ParseStream| {
        if let Some(_kw) = input.parse::<Option<optional>>()? {
            if output.optional.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[optional] attribute"));
            }
            output.optional = Some(Optional { original: attr });
            return Ok(());
        } else if let Some(_kw) = input.parse::<Option<untagged>>()? {
            if output.untagged.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[untagged] attribute"));
            }
            output.untagged = Some(Untagged { original: attr });
            return Ok(());
        } else if let Some(_kw) = input.parse::<Option<tag>>()? {
            let _eq_token: Token![=] = input.parse()?;
            let lit_int: LitInt = input.parse()?;
            if output.tag.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[tag] attribute"));
            }
            output.tag = Some(Tag {
                original: attr,
                tag: lit_int,
            });
            return Ok(());
        }
        let lit_int: LitInt = input.parse()?;
        if output.tag.is_some() {
            return Err(Error::new_spanned(attr, "duplicate #[tag] attribute"));
        }
        output.tag = Some(Tag {
            original: attr,
            tag: lit_int,
        });
        Ok(())
    })
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
        if let Some(optional) = &self.optional {
            return Err(Error::new_spanned(
                optional.original,
                "#[optional] at an invalid position",
            ));
        }
        Ok(())
    }

    pub fn disallow_untagged(&self) -> Result<()> {
        if let Some(untagged) = &self.untagged {
            return Err(Error::new_spanned(
                untagged.original,
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
