use std::ops::Not;

use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{meta::ParseNestedMeta, Attribute, DeriveInput, Fields};

macro_rules! define_attrs {
    ($name:ident { $(($ops:ident, $opname:ident)),* }) => {
        #[derive(Default, Debug, Clone, Copy)]
        struct $name {
            $($ops : bool),*
        }
        impl $name {
            fn from_syn(attrs: &Attribute) -> syn::Result<Self> {
                let mut me = Self::default();
                attrs.parse_nested_meta(|meta| {
                    let value = if let Ok(v) = meta.value() {
                        v.parse::<syn::LitBool>()?.value
                    } else {
                        true
                    };
                    if false {

                    } $( else if meta.path.is_ident(stringify!($opname)) { me.$ops = value; } )*
                    else {
                        return Err(meta.error("invalid argument"));
                    }
                    Ok(())
                })?;
                Ok(me)
            }
            fn from_attrs(attrs: &[Attribute]) -> Option<syn::Result<Self>> {
                for attr in attrs {
                    if attr.path().is_ident("sumtype") {
                        return Some(Self::from_syn(attr));
                    }
                }
                None
            }
        }
    };
}

define_attrs!(Attrs {
    (add_as, as),
    (add_into, into),
    (add_is, is),
    (ignore, ignore),
    (add_mut_as, mut_as)
});

pub fn derive_sum_type(input: DeriveInput) -> TokenStream {
    let syn::Data::Enum(data) = &input.data else {
        return syn::Error::new_spanned(input, "can only be placed on an enum").into_compile_error();
    };
    let kinds_ident = Ident::new(&format!("{}Kind", input.ident), Span::call_site());
    let input_ident = &input.ident;
    let vis = &input.vis;
    let attrs = Attrs::from_attrs(&input.attrs).unwrap_or(Ok(Attrs {
        add_as: true,
        add_into: true,
        add_is: true,
        ignore: false,
        add_mut_as: true,
    }));
    if let Err(e) = attrs {
        return e.to_compile_error();
    }
    let attrs = attrs.unwrap();
    let mut variant_names = Vec::new();
    let mut variants = Vec::new();
    let mut variant_tys = Vec::new();
    for variant in &data.variants {
        let attrs = Attrs::from_attrs(&variant.attrs).unwrap_or(Ok(attrs.clone()));
        if let Err(e) = attrs {
            return e.to_compile_error();
        }
        let attrs = attrs.unwrap();
        if attrs.ignore {
            continue;
        }
        variant_names.push(variant.ident.clone());
        variants.push((attrs, variant.ident.clone()));
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
    let lowercase_names = variants
        .iter()
        .map(|(a, v)| (a.clone(), v.to_string().to_case(Case::Snake)))
        .collect::<Vec<_>>();
    fn gen_names(
        names: &[(Attrs, impl std::fmt::Display)],
        prefix: &str,
        filter: impl Fn(&Attrs) -> bool,
    ) -> Vec<syn::Ident> {
        names
            .iter()
            .filter(|(a, _)| filter(a))
            .map(|(_, n)| Ident::new(&format!("{}_{}", prefix, n), Span::call_site()))
            .collect::<Vec<_>>()
    }
    let as_names = gen_names(&lowercase_names, "as", |a| a.add_as);
    let into_names = gen_names(&lowercase_names, "into", |a| a.add_into);
    let is_names = gen_names(&lowercase_names, "is", |a| a.add_is);
    let mut_as_names = gen_names(&lowercase_names, "as_mut", |a| a.add_mut_as);

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
                #vis fn #mut_as_names (&mut self) -> Option<&mut #variant_tys> {
                    match self {
                        Self::#variant_names (v) => Some(v),
                        _ => None,
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
