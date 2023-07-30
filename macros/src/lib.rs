use sum_type::Attrs;
use syn::{parse_macro_input, Attribute, DeriveInput};
mod sum_type;
#[proc_macro_derive(SumType, attributes(sumtype))]
pub fn derive_sum_type(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    sum_type::derive_sum_type(input).into()
}

#[proc_macro_attribute]
pub fn sumtype(
    attrs_ts: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut attrs = Attrs {
        add_as: true,
        add_into: true,
        add_is: true,
        ignore: false,
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
    let input = parse_macro_input!(item as syn::ItemEnum);
    sum_type::sumtype_attr(attrs, input).into()
}
