use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::DeriveInput;

#[derive(Default, Debug)]
pub struct Attrs {
    pub name: Option<String>,
    pub kind_fn: Option<String>,
    pub no_kind_fn: bool,
}

pub fn kinded_macro(attrs: Attrs, input: DeriveInput) -> syn::Result<TokenStream> {
    let syn::Data::Enum(data) = &input.data else {
        return Err(syn::Error::new_spanned(&input, "kinded only works on enums"));
    };
    let name = attrs
        .name
        .map(|s| Ident::new(&s, Span::mixed_site()))
        .unwrap_or_else(|| format_ident!("{}Kind", input.ident));
    let kind_fn = attrs
        .kind_fn
        .map(|s| Ident::new(&s, Span::mixed_site()))
        .unwrap_or_else(|| Ident::new("kind", Span::mixed_site()));
    let orig_input = &input.ident;
    let vis = &input.vis;
    let orig_variants = data.variants.iter().map(|v| {
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
    let kinds = data.variants.iter().map(|v| &v.ident).collect::<Vec<_>>();
    let kinds_fn = if attrs.no_kind_fn {
        None
    } else {
        Some(quote! {
            #vis fn #kind_fn (&self) -> #name {
                match self {
                    #(Self::#orig_variants => #name :: #kinds),*
                }
            }
        })
    };
    let o = quote! {
        #input
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
        #[allow(unused)]
        #vis enum #name {
            #(#kinds),*
        }
        #[automatically_derived]
        impl #orig_input {
            #kinds_fn
        }
    };
    Ok(o)
}
