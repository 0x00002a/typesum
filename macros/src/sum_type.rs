use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Fields};

pub fn derive_sum_type(input: DeriveInput) -> TokenStream {
    let syn::Data::Enum(data) = &input.data else {
        return syn::Error::new_spanned(input, "can only be placed on an enum").into_compile_error();
    };
    let kinds_ident = Ident::new(&format!("{}Kind", input.ident), Span::call_site());
    let input_ident = &input.ident;
    let vis = &input.vis;
    let mut variant_names = Vec::new();
    let mut variant_tys = Vec::new();
    for variant in &data.variants {
        variant_names.push(variant.ident.clone());
        let Fields::Unnamed(f) = &variant.fields else {
            return syn::Error::new_spanned(variant, "must be single variant").to_compile_error();
        };
        if f.unnamed.len() != 1 {
            return syn::Error::new_spanned(f, "must be single variant").to_compile_error();
        }
        variant_tys.push(f.unnamed.first().unwrap().ty.to_owned());
    }
    let kind_structs = variant_names
        .iter()
        .map(|v| Ident::new(&format!("{}{}Kind", input_ident, v), Span::call_site()))
        .collect::<Vec<_>>();
    let lowercase_names = variant_names
        .iter()
        .map(|v| v.to_string().to_case(Case::Snake))
        .collect::<Vec<_>>();
    let gen_names = |prefix| {
        lowercase_names
            .iter()
            .map(|n| Ident::new(&format!("{}_{}", prefix, n), Span::call_site()))
            .collect::<Vec<_>>()
    };
    let as_names = gen_names("as");
    let into_names = gen_names("into");
    let is_names = gen_names("is");

    quote! {
        #vis enum #kinds_ident {
            #(#variant_names (#kind_structs)),*
        }
        #(
            #[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord)]
            #vis struct #kind_structs {}
        )*
        #[automatically_derived]
        impl #input_ident {
            #(
                #vis fn #as_names (&self) -> Option<&#variant_tys> {
                    match self {
                        Self::#variant_names (v) => Some(v),
                        _ => None,
                    }
                }

                #vis fn #into_names (self) -> Option<#variant_tys> {
                    match self {
                        Self::#variant_names (v) => Some(v),
                        _ => None,
                    }
                }

                #vis fn #is_names (&self) -> bool {
                    match self {
                        Self::#variant_names (v) => true,
                        _ => false,
                    }
                }
            )*
        }
        #[automatically_derived]
        impl SumType for #input_ident {
            type Kind = #kinds_ident;
        }
    }
}
