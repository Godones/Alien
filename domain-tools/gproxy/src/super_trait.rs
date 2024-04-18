use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{ItemTrait, TypeParamBound};

pub fn impl_supertrait(ident: Ident, trait_def: ItemTrait) -> TokenStream {
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
                                        self.domain.read().handle_irq()
                                    }
                                }
                            );
                            code.push(device_base)
                        }
                        "Basic" => {
                            let basic = quote!(
                                impl Basic for #ident{
                                    fn is_active(&self)->bool{
                                        self.domain.read().is_active()
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
