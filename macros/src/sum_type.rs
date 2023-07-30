use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, DeriveInput, Fields};
macro_rules! define_attrs {
    ($name:ident { $(($ops:ident, $opname:ident)),* }) => {
        #[derive(Default, Debug, Clone, Copy)]
        pub struct $name {
            $(pub $ops : bool),*
        }
        impl $name {
            pub fn add_syn(&mut self, meta: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
                let value = if let Ok(v) = meta.value() {
                    v.parse::<syn::LitBool>()?.value
                } else {
                    true
                };
                if false {
                } $( else if meta.path.is_ident(stringify!($opname)) { self.$ops = value; } )*
                else {
                    return Err(meta.error("invalid argument"));
                }
                Ok(())
            }
            fn add_scope(mut self, attrs: &[Attribute]) -> syn::Result<Self> {
                for attr in attrs {
                    if attr.path().is_ident("sumtype") {
                        attr.parse_nested_meta(|meta| {
                            self.add_syn(&meta)?;
                            Ok(())
                        })?;
                        return Ok(self);
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
    (add_try_into, try_into),
    (add_try_into_impl, impl_try_into),
    (add_try_as, try_as),
    (add_try_as_mut, try_as_mut),
    (add_from_impl, from)
});
fn generate_conv_option<'a>(
    vis: &'a syn::Visibility,
    prefix: Option<TokenStream>,
) -> impl FnOnce(&[&Ident], &[&syn::Type], &[Ident]) -> TokenStream + 'a {
    move |variants: &[&Ident], tys: &[&syn::Type], names: &[Ident]| {
        quote! {
            #(
                #vis fn #names (#prefix self) -> Option<#prefix #tys> {
                    match self {
                        Self::#variants (v) => Some(v),
                        _ => None,
                    }
                }
            )*
        }
    }
}

fn generate_conv_try<'a>(
    vis: &'a syn::Visibility,
    input_ident: &'a Ident,
    prefix: Option<TokenStream>,
) -> impl FnOnce(&[&Ident], &[&syn::Type], &[Ident]) -> TokenStream + 'a {
    move |variants, tys, names| {
        quote! {
            #(
                #vis fn #names (#prefix self) -> Result<#prefix #tys, ::typesum::TryIntoError> {
                    match self {
                        Self::#variants (v) => Ok(v),
                        _ => Err(::typesum::TryIntoError::new(stringify!(#input_ident), stringify!(#tys))),
                    }
                }
            )*
        }
    }
}

fn gen_names<'a, 'b, A: 'a, B: 'a, C: 'a, R>(
    names: impl Iterator<Item = &'a (&'a (C, impl std::fmt::Display + 'a), (&'a A, &'a B))> + 'b,
    prefix: &str,
    filter: impl Fn(&C) -> bool,
    generate: impl FnOnce(&[&A], &[&B], &[Ident]) -> R,
) -> R
where
    'b: 'a,
{
    let (as_, bs, is) = names
        .filter(|((a, _), _)| filter(a))
        .map(|((_, n), (a, b))| {
            (
                a,
                b,
                Ident::new(&format!("{}_{}", prefix, n), Span::call_site()),
            )
        })
        .fold(
            (Vec::new(), Vec::new(), Vec::new()),
            |(mut as_, mut bs, mut is), (a, b, i)| {
                as_.push(*a);
                bs.push(*b);
                is.push(i);
                (as_, bs, is)
            },
        );
    generate(as_.as_slice(), bs.as_slice(), is.as_slice())
}
pub fn sumtype_attr(mut attrs: Attrs, input: syn::ItemEnum) -> TokenStream {
    let input_ident = &input.ident;
    let vis = &input.vis;
    let tys = &input.generics;
    attrs.add_try_into_impl &= input.generics.type_params().next().is_none();
    let mut variant_names = Vec::new();
    let mut variants = Vec::new();
    let mut variant_tys = Vec::new();
    for variant in &input.variants {
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
    let lowercase_names = variants
        .iter()
        .map(|(a, v)| (a.clone(), v.to_string().to_case(Case::Snake)))
        .collect::<Vec<_>>();
    let variants_zipped = lowercase_names
        .iter()
        .zip(variant_names.iter().zip(variant_tys.iter()))
        .collect::<Vec<_>>();
    let as_names = gen_names(
        variants_zipped.iter(),
        "as",
        |a| a.add_as,
        generate_conv_option(vis, Some(quote! { & })),
    );
    let into_names = gen_names(
        variants_zipped.iter(),
        "into",
        |a| a.add_into,
        generate_conv_option(vis, None),
    );
    let is_names = gen_names(
        variants_zipped.iter(),
        "is",
        |a| a.add_is,
        |variants, _, names| {
            quote! {
                #(
                    #vis fn #names (&self) -> bool {
                        match self {
                            Self::#variants (v) => true,
                            _ => false,
                        }
                    }
                )*
            }
        },
    );
    let mut_as_names = gen_names(
        variants_zipped.iter(),
        "as_mut",
        |a| a.add_mut_as,
        generate_conv_option(vis, Some(quote! { &mut })),
    );
    let try_into_names = gen_names(
        variants_zipped.iter(),
        "try_into",
        |a| a.add_try_into,
        generate_conv_try(vis, input_ident, None),
    );

    let try_as_impls = gen_names(
        variants_zipped.iter(),
        "try_as",
        |a| a.add_try_as,
        generate_conv_try(vis, input_ident, Some(quote! { & })),
    );

    let try_as_mut_impls = gen_names(
        variants_zipped.iter(),
        "try_as_mut",
        |a| a.add_try_as,
        generate_conv_try(vis, input_ident, Some(quote! { &mut })),
    );
    let try_intos = variants
        .iter()
        .zip(variant_tys.iter())
        .filter(|((a, _), _)| a.add_try_into_impl)
        .map(|((_, v), ty)| (ty, v))
        .collect::<Vec<_>>();
    let try_intos_idents = try_intos.iter().map(|(_, r)| r).collect::<Vec<_>>();
    let try_intos_tys = try_intos.iter().map(|(t, _)| t);
    let from_impls = variants_zipped
        .iter()
        .filter(|((a, _), _)| a.add_from_impl)
        .map(|((_, _), (v, t))| {
            quote! {
               #[automatically_derived]
               impl #tys From<#t> for #input_ident #tys {
                   fn from(value: #t) -> Self {
                       Self::#v (value)
                   }
               }
            }
        });

    let stripped_variants = variants
        .iter()
        .zip(variant_tys.iter())
        .map(|((_, n), t)| quote! { #n (#t) });

    let input_stripped = quote! {
       #vis enum #input_ident #tys {
           #(#stripped_variants),*
       }
    };

    quote! {
        #input_stripped
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
            #try_into_names
            #mut_as_names
            #as_names
            #into_names
            #is_names
            #try_as_impls
            #try_as_mut_impls
        }
        #(#from_impls)*
    }
}

pub fn derive_sum_type(input: DeriveInput) -> TokenStream {
    todo!()
}
