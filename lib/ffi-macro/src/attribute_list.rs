use syn::{
    Meta, Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
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
