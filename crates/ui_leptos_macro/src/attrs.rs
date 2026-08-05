use std::collections::HashMap;
use syn::{
    Ident, LitStr, Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

struct Pair {
    key: Ident,
    value: LitStr,
}

impl Parse for Pair {
    fn parse(input: ParseStream) -> Result<Self> {
        let key: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        let value: LitStr = input.parse()?;
        Ok(Pair { key, value })
    }
}

pub struct Attrs(pub HashMap<String, String>);

impl Attrs {
    pub fn get(&self, k: &str) -> Option<&String> {
        self.0.get(k)
    }
}

impl Parse for Attrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let list = Punctuated::<Pair, Token![,]>::parse_terminated(input)?;
        let map = list
            .iter()
            .map(|x| (x.key.to_string(), x.value.value()))
            .collect();
        Ok(Attrs(map))
    }
}