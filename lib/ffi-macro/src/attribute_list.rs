use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Meta, Result, Token,
};

pub struct AttributeList {
    variables: Vec<Meta>,
}

impl AttributeList {
    pub fn variables(&self) -> impl Iterator<Item = &Meta> {
        self.variables.iter()
    }
}

impl Parse for AttributeList {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            variables: Punctuated::<Meta, Token![,]>::parse_terminated(input)?
                .into_iter()
                .collect(),
        })
    }
}
