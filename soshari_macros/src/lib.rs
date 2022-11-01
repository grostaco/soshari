use proc_macro::TokenStream;

use proc_macro2::{Ident, Span, TokenTree};
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn generate_adjectives(stream: TokenStream) -> TokenStream {
    let stream = parse_macro_input!(stream as proc_macro2::TokenStream);

    let adjectives = stream
        .into_iter()
        .filter_map(|tree| {
            if let TokenTree::Ident(ident) = tree {
                Some(ident.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let adjectives_upper_ident = adjectives
        .iter()
        .map(|adjective| Ident::new(&adjective.to_uppercase(), Span::call_site()))
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
