use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::ItemEnum;

#[derive(Default, Debug)]
pub struct Attrs {
    pub name: Option<String>,
}

pub fn kinded_macro(attrs: Attrs, input: ItemEnum) -> syn::Result<TokenStream> {
    let name = attrs
        .name
        .map(|s| Ident::new(&s, Span::mixed_site()))
        .unwrap_or_else(|| format_ident!("{}Kind", input.ident));
    let orig_input = &input.ident;
    let vis = &input.vis;
    let orig_variants = input.variants.iter().map(|v| {
        let fields = match v.fields {
            syn::Fields::Named(_) => Some(quote! { { .. } }),
            syn::Fields::Unnamed(_) => Some(quote! {  (..) }),
            syn::Fields::Unit => None,
        };
        let ident = &v.ident;

        quote! {
            #ident #fields
        }
    });
    let kinds = input.variants.iter().map(|v| &v.ident).collect::<Vec<_>>();
    let o = quote! {
        #input
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
        #vis enum #name {
            #(#kinds),*
        }
        impl #orig_input {
            #vis fn kind(&self) -> #name {
                match self {
                    #(Self::#orig_variants => #name :: #kinds),*
                }
            }
        }
    };
    Ok(o)
}
