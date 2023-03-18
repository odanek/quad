use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Error, Fields, FieldsNamed};

pub fn derive_bundle(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let named_fields = match get_named_struct_fields(&ast.data) {
        Ok(fields) => &fields.named,
        Err(e) => return e.into_compile_error().into(),
    };

    let field = named_fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect::<Vec<_>>();
    let field_type = named_fields
        .iter()
        .map(|field| &field.ty)
        .collect::<Vec<_>>();

    let mut field_component_ids = Vec::new();
    let mut field_get_components = Vec::new();
    let mut field_from_components = Vec::new();
    for (field_type, field) in
        field_type.iter().zip(field.iter())
    {
        field_component_ids.push(quote! {
            component_ids.push(components.get_or_insert::<#field_type>());
        });
        field_get_components.push(quote! {
            func((&mut self.#field as *mut #field_type).cast::<u8>());
            std::mem::forget(self.#field);
        });
        field_from_components.push(quote! {
            #field: func().cast::<#field_type>().read(),
        });
    }
    let field_len = field.len();
    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let struct_name = &ast.ident;

    TokenStream::from(quote! {
        /// SAFE: ComponentId is returned in field-definition-order. [from_components] and [get_components] use field-definition-order
        unsafe impl #impl_generics ::quad::ecs::Bundle for #struct_name #ty_generics #where_clause {
            fn component_ids(
                components: &mut ::quad::ecs::Components,
            ) -> Vec<::quad::ecs::ComponentId> {
                let mut component_ids = Vec::with_capacity(#field_len);
                #(#field_component_ids)*
                component_ids
            }

            #[allow(unused_variables, unused_mut, non_snake_case)]
            unsafe fn from_components(mut func: impl FnMut() -> *mut u8) -> Self {
                Self {
                    #(#field_from_components)*
                }
            }

            #[allow(unused_variables, unused_mut, forget_copy, forget_ref, clippy::forget_non_drop)]
            fn get_components(mut self, mut func: impl FnMut(*mut u8)) {
                #(#field_get_components)*
            }
        }
    })
}

pub fn get_named_struct_fields(data: &syn::Data) -> syn::Result<&FieldsNamed> {
    match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => Ok(fields),
        _ => Err(Error::new(
            // This deliberately points to the call site rather than the structure
            // body; marking the entire body as the source of the error makes it
            // impossible to figure out which `derive` has a problem.
            Span::call_site().into(),
            "Only structs with named fields are supported",
        )),
    }
}
