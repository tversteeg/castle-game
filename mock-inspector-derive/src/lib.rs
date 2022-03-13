use proc_macro::TokenStream;

/// Add a proc macro for [`Inspectable`] that doesn't do anything.
///
/// Used when the "inspector" feature flag is disabled.
#[proc_macro_derive(Inspectable, attributes(inspectable))]
pub fn derive_helper_attr(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}
