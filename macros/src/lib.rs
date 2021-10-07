use proc_macro::TokenStream;

mod component;

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    component::derive_component(input)
}
