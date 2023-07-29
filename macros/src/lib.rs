use syn::{parse_macro_input, Attribute, DeriveInput};
mod sum_type;
mod types;
#[proc_macro_derive(SumType, attributes(sumtype))]
pub fn derive_sum_type(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    sum_type::derive_sum_type(input).into()
}

#[proc_macro_attribute]
pub fn sumtype(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attrs = parse_macro_input!(attrs with Attribute::parse_outer);
    let input = parse_macro_input!(item as syn::ItemEnum);
    sum_type::sumtype_attr(&attrs, input).into()
}
