use crate::children::Children;
use crate::element_attribute::ElementAttribute;
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream, Result};

pub type Attributes = HashSet<ElementAttribute>;

pub struct ElementAttributes {
    pub attributes: Attributes,
}

impl ElementAttributes {
    pub fn new(attributes: Attributes) -> Self {
        Self { attributes }
    }

    pub fn for_custom_element<'c>(
        &self,
        children: &'c Children,
    ) -> CustomElementAttributes<'_, 'c> {
        CustomElementAttributes {
            attributes: &self.attributes,
            children,
        }
    }

    pub fn for_simple_element(&self) -> SimpleElementAttributes<'_> {
        SimpleElementAttributes {
            attributes: &self.attributes,
        }
    }
}

impl Parse for ElementAttributes {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attributes: HashSet<ElementAttribute> = HashSet::new();
        while input.peek(syn::Ident) {
            if let Ok(attribute) = input.parse::<ElementAttribute>() {
                if attributes.contains(&attribute) {
                    let error_message = format!(
                        "There is a previous definition of the {} attribute",
                        attribute.ident()
                    );
                    attribute
                        .ident()
                        .span()
                        .unwrap()
                        .warning(error_message)
                        .emit();
                }
                attributes.insert(attribute);
            }
        }
        Ok(ElementAttributes::new(attributes))
    }
}

pub struct CustomElementAttributes<'a, 'c> {
    attributes: &'a Attributes,
    children: &'c Children,
}

impl<'a, 'c> ToTokens for CustomElementAttributes<'a, 'c> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut attrs: Vec<_> = self
            .attributes
            .iter()
            .map(|attribute| {
                let ident = attribute.ident();
                let value = attribute.value_tokens();

                quote! {
                    #ident: #value
                }
            })
            .collect();

        if self.children.len() > 0 {
            let children_tuple = self.children.as_option_of_tuples_tokens();
            attrs.push(quote! {
                children: #children_tuple
            });
        }

        let quoted = if attrs.len() == 0 {
            quote!()
        } else {
            quote!({ #(#attrs),* })
        };

        quoted.to_tokens(tokens);
    }
}

pub struct SimpleElementAttributes<'a> {
    attributes: &'a Attributes,
}

impl<'a> ToTokens for SimpleElementAttributes<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.attributes.is_empty() {
            quote!(None).to_tokens(tokens);
        } else {
            let attrs: Vec<_> = self
                .attributes
                .iter()
                .map(|attribute| {
                    let ident = attribute.ident();
                    let value = attribute.value_tokens();

                    quote! {
                        hm.insert(stringify!(#ident), #value);
                    }
                })
                .collect();

            let hashmap_declaration = quote! {{
                let mut hm = std::collections::HashMap::<&str, &str>::new();
                #(#attrs)*
                Some(hm)
            }};

            hashmap_declaration.to_tokens(tokens);
        }
    }
}