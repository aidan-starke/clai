use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn db_test(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_body = &input_fn.block;
    let is_async = input_fn.sig.asyncness.is_some();

    let expanded = if is_async {
        quote! {
            #[tokio::test]
            async fn #fn_name() {
                let _temp_db = common::setup_test_db();
                #fn_body
            }
        }
    } else {
        quote! {
            #[test]
            fn #fn_name() {
                let _temp_db = common::setup_test_db();
                #fn_body
            }
        }
    };

    TokenStream::from(expanded)
}
