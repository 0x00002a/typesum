use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{token::Not, Attribute, Fields};

#[derive(Clone, Copy)]
struct FullVariant<'a> {
    inner: &'a syn::Variant,
}
impl<'a> FullVariant<'a> {
    fn name(&self) -> &'a Ident {
        &self.inner.ident
    }
    fn match_pat(&self) -> impl ToTokens {
        let name = self.name();
        match &self.inner.fields {
            Fields::Named(_) => quote! {  #name { .. } },
            Fields::Unnamed(_) => quote! {  #name (..) },
            Fields::Unit => quote! {  #name },
        }
    }
}
fn bucketise_by<A, B>(
    mut f: impl FnMut(&A, &A) -> bool,
    input: impl Iterator<Item = (A, B)>,
) -> Vec<(A, Vec<B>)> {
    let mut buckets: Vec<(A, Vec<B>)> = Vec::new();
    for (a, b) in input {
        if let Some(bucket) = buckets.iter_mut().find(|(ba, _)| f(ba, &a)).map(|(_, b)| b) {
            bucket.push(b);
        } else {
            buckets.push((a, vec![b]));
        }
    }
    buckets
}

macro_rules! define_attrs {
    ($name:ident { $(($ops:ident, $opname:ident)),* }) => {
        #[derive(Default, Debug, Clone, Copy)]
        pub struct $name {
            $(pub $ops : bool),*
        }
        impl $name {
            pub fn add_syn(&mut self, meta: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
                const ERR: &'static str = concat!("property not recognised. must be one of: ", $(concat!(stringify!($opname), " ")),*);
                if meta.path.is_ident("only") {
                    let attr: syn::Ident = meta.value()?.parse()?;
                    let attr = attr.to_string();
                    $(self.$ops = stringify!($opname) == &attr;)*
                    if self.all_false() {
                        return Err(meta.error(ERR));
                    }
                    return Ok(());
                }
                let value = if let Ok(v) = meta.value() {
                    v.parse::<syn::LitBool>()?.value
                } else {
                    true
                };
                if meta.path.is_ident("ignore") {
                    $(self.$ops = false;)*
                } else if meta.path.is_ident("all") {
                    $(self.$ops = value;)*
                }
                $( else if meta.path.is_ident(stringify!($opname)) { self.$ops = value; } )*
                else {
                    return Err(meta.error(ERR));
                }
                Ok(())
            }
            pub fn all_false(&self) -> bool {
                !(false $(|| self.$ops)*)
            }

            pub fn all_false_but_is(&self) -> bool {
                if self.all_false() {
                    false
                } else {
                    let mut me = self.clone();
                    me.add_is = false;
                    me.all_false()
                }
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
    (add_mut_as, as_mut),
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
fn generate_failed_matches<'a>(
    variants: &[FullVariant<'a>],
    input_ident: &Ident,
    wanted: impl ToTokens,
) -> TokenStream {
    let patterns = variants.iter().map(|v| v.match_pat());
    let names = variants.iter().map(|v| v.name());
    quote! {
        #(
            #input_ident :: #patterns => Err(::typesum::TryIntoError::new(stringify!(#input_ident), stringify!(#names), #wanted))
        ),*
    }
}

/// Generate all the match blocks (match `on` { ... }) where each
/// variant is matched and all others are errors
fn generate_try_match_blocks<'a>(
    variants: &'a [&'a Ident],
    input_ident: &'a Ident,
    all_variants: &'a [FullVariant],
) -> impl Iterator<Item = TokenStream> + 'a {
    variants.iter().map(move |v| {
        let failed = generate_failed_matches(all_variants, input_ident, quote! { stringify!(#v) });
        quote! {
            match self {
                #input_ident :: #v(v) => Ok(v),
                #failed
            }
        }
    })
}

fn generate_conv_try<'a>(
    vis: &'a syn::Visibility,
    input_ident: &'a Ident,
    prefix: Option<TokenStream>,
    all_variants: &'a [FullVariant],
    input_tys: &'a syn::Generics,
) -> impl FnOnce(&[&Ident], &[&syn::Type], &[Ident]) -> TokenStream + 'a {
    move |variants, tys, names| {
        let blocks = generate_try_match_blocks(variants, input_ident, all_variants);
        quote! {
            #(
                #vis fn #names (#prefix self) -> ::core::result::Result<#prefix #tys, ::typesum::TryIntoError<#input_ident #input_tys>> {
                    #blocks
                }
            )*
        }
    }
}

fn gen_names<'a, 'b, A: 'a, B: 'a, C: 'a, R>(
    names: impl Iterator<Item = &'a (&'a (C, impl std::fmt::Display + 'a), (&'a A, &'a B))> + 'b,
    prefix: &str,
    suffix: Option<&str>,
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
                Ident::new(
                    &format!(
                        "{}_{}{}",
                        prefix,
                        n,
                        suffix
                            .map(|s| format!("_{s}"))
                            .unwrap_or_else(|| String::new())
                    ),
                    Span::call_site(),
                ),
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
const EXPLICITLY_DISABLE_FROM_MSG: &str = "You need to explicitly disable the ones you don't want with #[sumtype(from = false)]. See the docs on #[sumtype] for more information";

pub fn sumtype_attr(mut attrs: Attrs, input: syn::DeriveInput) -> TokenStream {
    let syn::Data::Enum(data) = &input.data else {
        return syn::Error::new_spanned(&input, "sumtype can only act on enums").to_compile_error();
    };
    let input_ident = &input.ident;
    let vis = &input.vis;
    let tys = &input.generics;
    attrs.add_try_into_impl &= input.generics.type_params().next().is_none();
    let mut variant_names = Vec::new();
    let mut variants = Vec::new();
    let mut variant_tys = Vec::new();
    let all_variant_matches = data
        .variants
        .iter()
        .map(|v| FullVariant { inner: v })
        .collect::<Vec<_>>();
    for variant in &data.variants {
        let attrs = attrs.add_scope(&variant.attrs);
        if let Err(e) = attrs {
            return e.to_compile_error();
        }
        let attrs = attrs.unwrap();
        if attrs.all_false() {
            continue;
        }
        variant_names.push(variant.ident.clone());
        variants.push((attrs, variant.ident.clone()));
        if attrs.all_false_but_is() {
            variant_tys.push(syn::Type::Never(syn::TypeNever {
                bang_token: Not::default(),
            }));
            continue;
        }
        let Fields::Unnamed(f) = &variant.fields else {
            return syn::Error::new_spanned(variant, "must be single variant").to_compile_error();
        };
        if f.unnamed.len() != 1 {
            return syn::Error::new_spanned(f, "must be single variant").to_compile_error();
        }
        variant_tys.push(f.unnamed.first().unwrap().ty.to_owned());
    }
    if variants.is_empty() {
        return quote! {
            compile_error!("this sumtype annotation won't do anything, try adding some options like #[sumtype(all = false, is = true)]");
        }.into();
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
        None,
        |a| a.add_as,
        generate_conv_option(vis, Some(quote! { & })),
    );
    let into_names = gen_names(
        variants_zipped.iter(),
        "into",
        None,
        |a| a.add_into,
        generate_conv_option(vis, None),
    );
    let is_names = all_variant_matches
        .iter()
        .zip(lowercase_names.iter())
        .filter(|(_, (a, _))| a.add_is)
        .map(|(f, (_, name))| {
            let name = format_ident!("is_{name}");
            let case = f.match_pat();
            quote! {
                #vis fn #name (&self) -> bool {
                    match self {
                        Self::#case => true,
                        _ => false,
                    }
                }
            }
        });
    let mut_as_names = gen_names(
        variants_zipped.iter(),
        "as",
        Some("mut"),
        |a| a.add_mut_as,
        generate_conv_option(vis, Some(quote! { &mut })),
    );
    let try_into_names = gen_names(
        variants_zipped.iter(),
        "try_into",
        None,
        |a| a.add_try_into,
        generate_conv_try(vis, input_ident, None, &all_variant_matches, &tys),
    );

    let try_as_impls = gen_names(
        variants_zipped.iter(),
        "try_as",
        None,
        |a| a.add_try_as,
        generate_conv_try(
            vis,
            input_ident,
            Some(quote! { & }),
            &all_variant_matches,
            &tys,
        ),
    );

    let try_as_mut_impls = gen_names(
        variants_zipped.iter(),
        "try_as",
        Some("mut"),
        |a| a.add_try_as,
        generate_conv_try(
            vis,
            input_ident,
            Some(quote! { &mut }),
            &all_variant_matches,
            &tys,
        ),
    );
    let try_intos = variants
        .iter()
        .zip(variant_tys.iter())
        .filter(|((a, _), _)| a.add_try_into_impl)
        .map(|((_, v), ty)| (ty, v))
        .collect::<Vec<_>>();
    let from_candidates = variants_zipped
        .iter()
        .filter(|((a, _), _)| a.add_from_impl)
        .collect::<Vec<_>>();
    let mut seen_from = Vec::new();
    let mut seen_generics = 0;
    for (_, (_, candidate)) in &from_candidates {
        if let syn::Type::Path(p) = candidate {
            if tys
                .type_params()
                .find(|t| p.path.is_ident(&t.ident))
                .is_some()
            {
                seen_generics += 1;
                if seen_generics > 1 {
                    return syn::Error::new_spanned(candidate, format!("Multiple generic's with From implementations found, these will conflict. {EXPLICITLY_DISABLE_FROM_MSG}")).to_compile_error();
                }
            }
        }

        if seen_from.contains(candidate) {
            return syn::Error::new_spanned(
                candidate,
                "Multiple valid From candidates found. {EXPLICITLY_DISABLE_FROM_MSG}",
            )
            .to_compile_error();
        } else {
            seen_from.push(&candidate);
        }
    }
    let from_impls = from_candidates.into_iter().map(|((_, _), (v, t))| {
        quote! {
           #[automatically_derived]
           impl #tys ::core::convert::From<#t> for #input_ident #tys {
               fn from(value: #t) -> Self {
                   Self::#v (value)
               }
           }
        }
    });

    let mut minput = input.clone();
    let syn::Data::Enum(minput_data) = &mut minput.data else {
        unreachable!()
    };
    let filt = |a: &syn::Attribute| a.meta.path().is_ident("sumtype");
    for v in &mut minput_data.variants {
        let to_remove = v.attrs.iter().filter(|a| filt(a)).count();
        for _ in 0..to_remove {
            let pos = v.attrs.iter().position(filt).unwrap();
            v.attrs.swap_remove(pos);
        }
    }
    let input_stripped = quote! {
        #minput
    };
    let try_into_impls = bucketise_by(|l, r| l == r, try_intos.into_iter())
        .into_iter()
        .map(|(ty, idents)| {
            let name = idents
                .iter()
                .map(|i| i.to_string())
                .reduce(|xs, x| format!("{xs} | {x}"))
                .unwrap();
            let failed = generate_failed_matches(&all_variant_matches, input_ident, &name);
            quote! {
                #[automatically_derived]
                impl #tys ::core::convert::TryInto<#ty> for #input_ident #tys {
                    type Error = ::typesum::TryIntoError<#input_ident #tys>;
                    fn try_into(self) -> ::core::result::Result<#ty, Self::Error> {
                        match self {
                            #(Self:: #idents (v) => Ok(v),)*
                            #failed
                        }
                    }
                }
            }
        });

    quote! {
        #input_stripped

        #(#try_into_impls)*

        #[automatically_derived]
        impl #tys #input_ident #tys {
            #try_into_names
            #mut_as_names
            #as_names
            #into_names
            #try_as_impls
            #try_as_mut_impls
            #(#is_names)*
        }
        #(#from_impls)*
    }
}
