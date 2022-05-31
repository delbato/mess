use proc_macro::TokenStream;

#[proc_macro]
pub fn mess_function(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro]
pub fn mess_container(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}