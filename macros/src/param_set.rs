use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote};
use syn::{
    Ident, Index,
};

fn get_idents(fmt_string: fn(usize) -> String, count: usize) -> Vec<Ident> {
    (0..count)
        .map(|i| Ident::new(&fmt_string(i), Span::call_site()))
        .collect::<Vec<Ident>>()
}

pub fn impl_param_set(_input: TokenStream) -> TokenStream {
    let mut tokens = TokenStream::new();
    let max_params = 8;
    let params = get_idents(|i| format!("P{}", i), max_params);
    let params_fetch = get_idents(|i| format!("PF{}", i), max_params);
    let metas = get_idents(|i| format!("m{}", i), max_params);
    let mut param_fn_muts = Vec::new();
    for (i, param) in params.iter().enumerate() {
        let fn_name = Ident::new(&format!("p{}", i), Span::call_site());
        let index = Index::from(i);
        param_fn_muts.push(quote! {
            pub fn #fn_name<'a>(&'a mut self) -> <#param::Fetch as SystemParamFetch<'a, 'a>>::Item {
                // SAFE: systems run without conflicts with other systems.
                // Conflicting params in ParamSet are not accessible at the same time
                // ParamSets are guaranteed to not conflict with other SystemParams
                unsafe {
                    <#param::Fetch as SystemParamFetch<'a, 'a>>::get_param(&mut self.param_states.#index, &self.system_meta, self.world, self.change_tick)
                }
            }
        });
    }

    for param_count in 1..=max_params {
        let param = &params[0..param_count];
        let param_fetch = &params_fetch[0..param_count];
        let meta = &metas[0..param_count];
        let param_fn_mut = &param_fn_muts[0..param_count];
        tokens.extend(TokenStream::from(quote! {
            impl<'w, 's, #(#param: SystemParam,)*> SystemParam for ParamSet<'w, 's, (#(#param,)*)>
            {
                type Fetch = ParamSetState<(#(#param::Fetch,)*)>;
            }

            // SAFE: All parameters are constrained to ReadOnlyFetch, so World is only read

            unsafe impl<#(#param_fetch: for<'w1, 's1> SystemParamFetch<'w1, 's1>,)*> ReadOnlySystemParamFetch for ParamSetState<(#(#param_fetch,)*)>
            where #(#param_fetch: ReadOnlySystemParamFetch,)*
            { }

            // SAFE: Relevant parameter ComponentId and ArchetypeComponentId access is applied to SystemMeta. If any ParamState conflicts
            // with any prior access, a panic will occur.

            unsafe impl<#(#param_fetch: for<'w1, 's1> SystemParamFetch<'w1, 's1>,)*> SystemParamState for ParamSetState<(#(#param_fetch,)*)>
            {
                fn new(world: &mut World, system_meta: &mut SystemMeta) -> Self {
                    #(
                        // Pretend to add each param to the system alone, see if it conflicts
                        let mut #meta = system_meta.clone();
                        #meta.component_access.clear();
                        #param_fetch::new(world, &mut #meta);
                        let #param = #param_fetch::new(world, &mut system_meta.clone());
                    )*
                    #(
                        system_meta
                            .component_access
                            .extend(#meta.component_access);
                    )*
                    ParamSetState((#(#param,)*))
                }

                fn update(&mut self, world: &World, system_meta: &mut SystemMeta) {
                    let (#(#param,)*) = &mut self.0;
                    #(
                        #param.update(world, system_meta);
                    )*
                }
            }

            impl<'w, 's, #(#param_fetch: for<'w1, 's1> SystemParamFetch<'w1, 's1>,)*> SystemParamFetch<'w, 's> for ParamSetState<(#(#param_fetch,)*)>
            {
                type Item = ParamSet<'w, 's, (#(<#param_fetch as SystemParamFetch<'w, 's>>::Item,)*)>;

                #[inline]
                unsafe fn get_param(
                    state: &'s mut Self,
                    system_meta: &SystemMeta,
                    world: &'w World,
                    change_tick: Tick,
                ) -> Self::Item {
                    ParamSet {
                        param_states: &mut state.0,
                        system_meta: system_meta.clone(),
                        world,
                        change_tick,
                    }
                }
            }

            impl<'w, 's, #(#param: SystemParam,)*> ParamSet<'w, 's, (#(#param,)*)>
            {
                #(#param_fn_mut)*
            }
        }));
    }

    tokens
}