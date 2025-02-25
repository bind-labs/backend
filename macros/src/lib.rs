use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(IntoRequest)]
pub fn derive_into_request(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let output = quote! {
        impl #name {
            pub fn into_request(self, method: http::Method, url: &str) -> http::Request<String> {
                http::Request::builder()
                    .uri(url)
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .method(method)
                    .body(serde_json::to_string(&self).unwrap())
                    .unwrap()
            }
        }
    };

    TokenStream::from(output)
}
