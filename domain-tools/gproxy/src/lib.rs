use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, ItemTrait, TraitItem, TraitItemFn, TypeParamBound};

#[proc_macro_attribute]
pub fn proxy(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let domain_name = parse_macro_input!(attr as Ident);
    let trait_def = parse_macro_input!(item as ItemTrait);
    let struct_def = def_struct(domain_name, trait_def.clone());
    quote!(
        #trait_def
        #struct_def
    )
    .into()
}

fn def_struct(ident: Ident, trait_def: ItemTrait) -> TokenStream {
    let trait_name = trait_def.ident.clone();
    let func_vec = trait_def.items.clone();

    let supertrait_code = impl_supertrait(ident.clone(), trait_def.clone());

    let func_code = impl_func(func_vec);
    let macro_ident = format!("gen_for_{}", trait_name);
    let macro_ident = Ident::new(&macro_ident, trait_name.span());

    let impl_ident = format!("impl_for_{}", trait_name);
    let impl_ident = Ident::new(&impl_ident, trait_name.span());

    quote::quote!(
        #[macro_export]
        macro_rules! #macro_ident {
            () => {
                #[derive(Debug)]
                pub struct #ident{
                    id:u64,
                    domain: alloc::boxed::Box<dyn  #trait_name>
                }
                impl #ident{
                    pub fn new(id:u64, domain: alloc::boxed::Box<dyn #trait_name>)->Self{
                        Self{
                            id,
                            domain
                        }
                    }
                }
                #supertrait_code
                impl #trait_name for #ident{
                    #(#func_code)*
                }
            };
        }
        #[macro_export]
        macro_rules! #impl_ident {
            ($name:ident) => {
                impl #trait_name for $name{
                    #(#func_code)*
                }
            }
        }

        // #macro_ident!(#ident);
    )
    .into()
}

fn impl_supertrait(ident: Ident, trait_def: ItemTrait) -> TokenStream {
    let supertraits = trait_def.supertraits.clone();
    let mut code = vec![];
    for supertrait in supertraits {
        match supertrait {
            TypeParamBound::Trait(trait_bound) => {
                let path = trait_bound.path.clone();
                let segments = path.segments;
                for segment in segments {
                    let trait_name = segment.ident.clone();
                    match trait_name.to_string().as_str() {
                        "DeviceBase" => {
                            let device_base = quote!(
                                impl DeviceBase for #ident{
                                    fn handle_irq(&self)->AlienResult<()>{
                                        if !self.is_active() {
                                            return Err(AlienError::DOMAINCRASH);
                                        }
                                        self.domain.handle_irq()
                                    }
                                }
                            );
                            code.push(device_base)
                        }
                        "Basic" => {
                            let basic = quote!(
                                impl Basic for #ident{
                                    fn is_active(&self)->bool{
                                        self.domain.is_active()
                                    }
                                }
                            );
                            code.push(basic)
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    quote::quote!(
        #(#code)*
    )
    .into()
}

fn impl_func(func_vec: Vec<TraitItem>) -> Vec<TokenStream> {
    let func_code: Vec<TokenStream> = func_vec
        .iter()
        .map(|item| match item {
            TraitItem::Fn(method) => {
                let func_code = impl_func_code(&method);
                func_code
            }
            _ => {
                panic!("item is not a function");
            }
        })
        .collect();
    func_code
}

fn impl_func_code(func: &TraitItemFn) -> TokenStream {
    let name = func.sig.ident.clone();
    let attr = func.attrs.clone();
    let sig = func.sig.clone();
    let input = sig.inputs.clone();
    let input_argv = input
        .iter()
        .skip(1)
        .map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => {
                let pat = pat_type.pat.as_ref();
                match pat {
                    syn::Pat::Ident(ident) => {
                        let name = ident.ident.clone();
                        name
                    }
                    _ => {
                        panic!("not a ident");
                    }
                }
            }
            _ => {
                panic!("not a typed");
            }
        })
        .collect::<Vec<Ident>>();

    match name.to_string().as_str() {
        "init" => {
            let token = quote!(
                #(#attr)*
                #sig{
                    self.domain.init(#(#input_argv),*)
                }
            );
            token
        }
        _ => {
            let token = quote!(
                #(#attr)*
                #sig{
                    if !self.is_active() {
                        return Err(AlienError::DOMAINCRASH);
                    }
                    self.domain.#name(#(#input_argv),*)
                }
            );
            token
        }
    }
}
