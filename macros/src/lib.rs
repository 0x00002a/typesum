use syn::{parse_macro_input, DeriveInput};

mod sum_type;
#[proc_macro_derive(SumType)]
pub fn derive_sum_type(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    sum_type::derive_sum_type(input).into()
}
