use std::ops::Not;

use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{meta::ParseNestedMeta, Attribute, DeriveInput, Fields};
macro_rules! define_attrs {
    ($name:ident { $(($ops:ident, $opname:ident)),* }) => {
        #[derive(Default, Debug, Clone, Copy)]
        struct $name {
            $($ops : bool),*
        }
        impl $name {
            fn add_syn(self, attrs: &Attribute) -> syn::Result<Self> {
                let mut me = self;
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
            fn add_scope(self, attrs: &[Attribute]) -> syn::Result<Self> {
                for attr in attrs {
                    if attr.path().is_ident("sumtype") {
                        return self.add_syn(attr);
                    }
                }
                Ok(self)
            }
        }
    };
}

define_attrs!(Attrs {
    (add_as, as),
    (add_into, into),
    (add_is, is),
    (ignore, ignore),
    (add_mut_as, mut_as),
    (add_try_into, try_into)
});

pub fn derive_sum_type(input: DeriveInput) -> TokenStream {
    let syn::Data::Enum(data) = &input.data else {
        return syn::Error::new_spanned(input, "can only be placed on an enum").into_compile_error();
    };
    let kinds_ident = Ident::new(&format!("{}Kind", input.ident), Span::call_site());
    let input_ident = &input.ident;
    let vis = &input.vis;
    let attrs = Attrs {
        add_as: true,
        add_into: true,
        add_is: true,
        ignore: false,
        add_mut_as: true,
        add_try_into: input.generics.type_params().next().is_none(),
    }
    .add_scope(&input.attrs);
    if let Err(e) = attrs {
        return e.to_compile_error();
    }
    let attrs = attrs.unwrap();
    let mut variant_names = Vec::new();
    let mut variants = Vec::new();
    let mut variant_tys = Vec::new();
    for variant in &data.variants {
        let attrs = attrs.add_scope(&variant.attrs);
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
    let try_intos = variants
        .iter()
        .zip(variant_tys.iter())
        .filter(|((a, _), _)| a.add_try_into)
        .map(|((_, v), ty)| (ty, v))
        .collect::<Vec<_>>();
    let try_intos_idents = try_intos.iter().map(|(_, r)| r).collect::<Vec<_>>();
    let try_intos_tys = try_intos.iter().map(|(t, _)| t);

    let tys = input.generics;
    quote! {
        #vis enum #kinds_ident {
            #(#variant_names (#kind_structs)),*
        }
        #(
            #[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord)]
            #vis struct #kind_structs {}
        )*
        #(
            #[automatically_derived]
            impl #tys TryInto<#try_intos_tys> for #input_ident #tys {
                type Error = ::typesum::TryIntoError;
                fn try_into(self) -> Result<#try_intos_tys, Self::Error> {
                    match self {
                        Self:: #try_intos_idents (v) => Ok(v),
                        _ => Err(::typesum::TryIntoError::new(stringify!(#input_ident), stringify!(#try_intos_tys))),
                    }
                }
            }
        )*
        #[automatically_derived]
        impl #tys #input_ident #tys {
            #(
                #vis fn #as_names (&self) -> Option<&#variant_tys> {
                    match self {
                        Self::#variant_names (v) => Some(v),
                        _ => None,
                    }
                }
            )*
            #(
                #vis fn #into_names (self) -> Option<#variant_tys> {
                    match self {
                        Self::#variant_names (v) => Some(v),
                        _ => None,
                    }
                }

            )*
            #(
                #vis fn #is_names (&self) -> bool {
                    match self {
                        Self::#variant_names (v) => true,
                        _ => false,
                    }
                }
            )*
            #(
                #vis fn #mut_as_names (&mut self) -> Option<&mut #variant_tys> {
                    match self {
                        Self::#variant_names (v) => Some(v),
                        _ => None,
                    }
                }
            )*
        }
        #[automatically_derived]
        impl #tys SumType for #input_ident #tys {
            type Kind = #kinds_ident;
        }
    }
}
