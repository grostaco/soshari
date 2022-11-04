use proc_macro::TokenStream;

use proc_macro2::{Ident, Span, TokenTree};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro]
pub fn generate_adjectives(stream: TokenStream) -> TokenStream {
    let stream = parse_macro_input!(stream as proc_macro2::TokenStream);

    let adjectives = stream
        .into_iter()
        .filter_map(|tree| {
            if let TokenTree::Ident(ident) = tree {
                Some(ident.to_string().replace('_', "-"))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let adjectives_upper_ident = adjectives
        .iter()
        .map(|adjective| {
            Ident::new(
                &adjective.replace('-', "_").to_uppercase(),
                Span::call_site(),
            )
        })
        .collect::<Vec<_>>();

    let bit_offset = 0..adjectives.len();
    quote! {
        const ADJECTIVES: &[&str] = &[
            #(#adjectives),*
        ];

        bitflags::bitflags! {
            #[derive(serde::Serialize, serde::Deserialize)]
            struct Adjectives: u64 {
                #(const #adjectives_upper_ident = 1 << #bit_offset;)*
            }
        }

        fn adjectives_to_bitflags(johari_adjectives: Vec<String>) -> Adjectives {
            let mut adjectives = Adjectives::empty();
            for adjective in johari_adjectives {
                match adjective.as_str() {
                    #(#adjectives => adjectives.set(Adjectives::#adjectives_upper_ident, true),)*
                    _ => {}
                }
            }
            adjectives
        }

        fn bitflags_to_adjectives(adjectives: Adjectives) -> Vec<&'static str> {
            let mut adjective_vec = Vec::new();

            #(if adjectives.contains(Adjectives::#adjectives_upper_ident) {
                adjective_vec.push(#adjectives);
            })*

            adjective_vec
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn adjectives(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as proc_macro2::TokenStream);

    let adjectives = attr
        .into_iter()
        .filter_map(|tree| {
            if let TokenTree::Ident(ident) = tree {
                Some(ident.to_string().replace('_', "-"))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let adjectives_upper_ident = adjectives
        .iter()
        .map(|adjective| {
            Ident::new(
                &adjective.replace('-', "_").to_uppercase(),
                Span::call_site(),
            )
        })
        .collect::<Vec<_>>();

    let derive = parse_macro_input!(item as DeriveInput);
    let ident = &derive.ident;
    let group_ident = Ident::new(&format!("{ident}Group"), Span::call_site());

    let bitmap_struct = Ident::new(&format!("{ident}Adjectives"), Span::call_site());
    let bit_offset = 0..adjectives.len();

    quote! {
        #derive

        struct #group_ident(Vec<#ident>);
        impl #group_ident {
            fn load<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
                Ok(Self(
                    serde_json::from_str(&std::fs::read_to_string(path)?).unwrap_or(Vec::new()),
                ))
            }

            fn dump<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
                serde_json::to_writer_pretty(
                    std::fs::OpenOptions::new().write(true).truncate(true).open(path)?,
                    &self.0,
                )?;
                Ok(())
            }

            fn get<U: Into<u64> + Copy>(&self, id: U) -> Option<&#ident> {
                self.0.iter().find(|n| n.id == id.into())
            }

            fn get_mut<U: Into<u64> + Copy>(&mut self, id: U) -> Option<&mut #ident> {
                self.0.iter_mut().find(|n| n.id == id.into())
            }

            fn push(&mut self, other: #ident) {
                match self.get_mut(other.id) {
                    Some(s) => s.adjectives = other.adjectives,
                    None => self.0.push(other),
                }
            }
        }

        bitflags::bitflags! {
            #[derive(serde::Serialize, serde::Deserialize, Default)]
            struct #bitmap_struct: u64 {
                #(const #adjectives_upper_ident = 1 << #bit_offset;)*
            }
        }

        impl From<Vec<String>> for #bitmap_struct {
            fn from(adjectives: Vec<String>) -> Self {
                let mut ret = Self::empty();
                for adjective in adjectives {
                    match adjective.as_str() {
                        #(#adjectives => ret.set(Self::#adjectives_upper_ident, true),)*
                        _ => {}
                    }
                }
                ret
            }
        }

        impl #bitmap_struct {
            pub fn as_adjectives(&self) -> Vec<&'static str> {
                let mut adjective_vec = Vec::new();

                #(if self.contains(Self::#adjectives_upper_ident) {
                    adjective_vec.push(#adjectives);
                })*

                adjective_vec
            }

            #[inline]
            pub fn adjectives() -> &'static [&'static str] {
                &[#(#adjectives),*]
            }
        }
    }
    .into()
}
