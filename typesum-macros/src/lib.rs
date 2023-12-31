use syn::parse_macro_input;
#[cfg(feature = "kinded")]
mod kinded;
#[cfg(feature = "sumtype")]
mod sum_type;

fn handle_syn_result(r: syn::Result<proc_macro2::TokenStream>) -> proc_macro::TokenStream {
    match r {
        Ok(v) => v.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[cfg(feature = "kinded")]
#[proc_macro_attribute]
pub fn kinded(
    attrs_ts: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut kind_attrs = kinded::Attrs::default();
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("name") {
            let value: syn::LitStr = meta.value()?.parse()?;
            kind_attrs.name.replace(value.value());
        } else if meta.path.is_ident("no_kind_fn") {
            kind_attrs.no_kind_fn = true;
        } else if meta.path.is_ident("kind_fn") {
            let value: syn::LitStr = meta.value()?.parse()?;
            kind_attrs.kind_fn.replace(value.value());
        } else {
            return Err(meta.error("invalid argument"));
        }
        Ok(())
    });
    parse_macro_input!(attrs_ts with parser);
    let item = parse_macro_input!(item as syn::DeriveInput);
    handle_syn_result(kinded::kinded_macro(kind_attrs, item))
}

#[cfg(feature = "sumtype")]
#[proc_macro_attribute]
pub fn sumtype(
    attrs_ts: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut attrs = sum_type::Attrs {
        add_as: true,
        add_into: true,
        add_is: true,
        add_mut_as: true,
        add_try_into: true,
        add_try_into_impl: false,
        add_try_as: true,
        add_try_as_mut: true,
        add_from_impl: true,
    };
    let parser = syn::meta::parser(|meta| {
        attrs.add_syn(&meta)?;
        Ok(())
    });
    parse_macro_input!(attrs_ts with parser);
    let input = parse_macro_input!(item as syn::DeriveInput);
    sum_type::sumtype_attr(attrs, input).into()
}
