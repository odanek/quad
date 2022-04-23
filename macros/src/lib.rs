use proc_macro::TokenStream;

mod component;
mod event;
mod resource;
mod bundle;
mod param_set;

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    component::derive_component(input)
}

#[proc_macro_derive(Event)]
pub fn derive_event(input: TokenStream) -> TokenStream {
    event::derive_event(input)
}

#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    resource::derive_resource(input)
}

#[proc_macro_derive(Bundle)]
pub fn derive_bundle(input: TokenStream) -> TokenStream {
    bundle::derive_bundle(input)
}

#[proc_macro]
pub fn impl_param_set(input: TokenStream) -> TokenStream {
    param_set::impl_param_set(input)
}