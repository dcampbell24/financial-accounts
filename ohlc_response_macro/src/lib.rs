#![feature(proc_macro_quote)]

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(OhlcResponseDerive)]
pub fn derive_ohlc_response(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl OhlcResponse for Response<#name> {
            fn result(&self) -> OhlcVec {
                OhlcVec {
                    ohlc: self.result.ohlc.clone(),
                    last: self.result.last,
                }
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
