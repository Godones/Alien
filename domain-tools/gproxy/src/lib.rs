mod super_trait;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, FnArg, ItemTrait, ReturnType, Token, TraitItem, TraitItemFn, Type,
};

use crate::super_trait::impl_supertrait;

struct Proxy {
    ident: Ident,
    #[allow(unused)]
    comma: Option<Token![,]>,
    source: Option<Type>,
}

impl Parse for Proxy {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        let comma: Option<Token![,]> = input.parse()?;
        match comma {
            Some(c) => {
                let ty = input.parse::<Type>()?;
                Ok(Proxy {
                    ident,
                    comma: Some(c),
                    source: Some(ty),
                })
            }
            None => Ok(Proxy {
                ident,
                comma: None,
                source: None,
            }),
        }
    }
}

#[proc_macro_attribute]
/// Whether to generate trampoline code for the function
pub fn recover(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = TokenStream::from(item);
    quote! (
        #item
    )
    .into()
}
#[proc_macro_attribute]
/// Do not check if the domain is active
pub fn no_check(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = TokenStream::from(item);
    quote! (
        #item
    )
    .into()
}

#[proc_macro_attribute]
pub fn proxy(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let proxy = parse_macro_input!(attr as Proxy);
    let trait_def = parse_macro_input!(item as ItemTrait);
    let struct_def = def_struct(proxy, trait_def.clone());
    quote!(
        #trait_def
        #struct_def
    )
    .into()
}

fn def_struct(proxy: Proxy, trait_def: ItemTrait) -> TokenStream {
    let trait_name = trait_def.ident.clone();
    let func_vec = trait_def.items.clone();

    let ident = proxy.ident.clone();
    let supertrait_code = impl_supertrait(ident.clone(), trait_def.clone());

    let (func_code, extern_func_code) =
        impl_func(func_vec, &trait_name, &ident, proxy.source.is_some());
    let macro_ident = format!("gen_for_{}", trait_name);
    let macro_ident = Ident::new(&macro_ident, trait_name.span());

    let impl_ident = format!("impl_for_{}", trait_name);
    let impl_ident = Ident::new(&impl_ident, trait_name.span());

    let resource_field = if proxy.source.is_some() {
        let ty = proxy.source.as_ref().unwrap();
        quote! (
            resource: Once<#ty>
        )
    } else {
        quote!()
    };

    let resource_init = if proxy.source.is_some() {
        quote! (
            resource: Once::new()
        )
    } else {
        quote!()
    };

    quote::quote!(
        #[macro_export]
        macro_rules! #macro_ident {
            () => {
                #[derive(Debug)]
                pub struct #ident{
                    id:AtomicU64,
                    domain: RwLock<alloc::boxed::Box<dyn #trait_name>>,
                    old_domain: Mutex<alloc::vec::Vec<alloc::boxed::Box<dyn #trait_name>>>,
                    domain_loader: Mutex<DomainLoader>,
                    #resource_field
                }
                impl #ident{
                    pub fn new(id:u64, domain: alloc::boxed::Box<dyn #trait_name>,domain_loader: DomainLoader)->Self{
                        Self{
                            id: AtomicU64::new(id),
                            domain: RwLock::new(domain),
                            old_domain: Mutex::new(alloc::vec::Vec::new()),
                            domain_loader: Mutex::new(domain_loader),
                            #resource_init
                        }
                    }
                }
                #supertrait_code


                impl #trait_name for #ident{
                    #(#func_code)*
                }

                #(#extern_func_code)*
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
    )
    .into()
}

fn impl_func(
    func_vec: Vec<TraitItem>,
    trait_name: &Ident,
    proxy_name: &Ident,
    has_resource: bool,
) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let mut func_codes = vec![];
    let mut extern_func_codes = vec![];
    func_vec.iter().for_each(|item| match item {
        TraitItem::Fn(method) => {
            let (func_code, extern_func_code) =
                impl_func_code(&method, trait_name, proxy_name, has_resource);
            func_codes.push(func_code);
            extern_func_codes.push(extern_func_code);
        }
        _ => {
            panic!("item is not a function");
        }
    });
    (func_codes, extern_func_codes)
}

fn impl_func_code(
    func: &TraitItemFn,
    trait_name: &Ident,
    proxy_name: &Ident,
    has_resource: bool,
) -> (TokenStream, TokenStream) {
    let has_recover = func
        .attrs
        .iter()
        .find(|attr| {
            let path = attr.path();
            path.is_ident("recover")
        })
        .is_some();

    let no_check = func
        .attrs
        .iter()
        .find(|attr| {
            let path = attr.path();
            path.is_ident("no_check")
        })
        .is_some();

    let name = func.sig.ident.clone();
    let mut attr = func.attrs.clone();

    attr.retain(|attr| {
        let path = attr.path();
        !path.is_ident("recover") && !path.is_ident("no_check")
    });

    let sig = func.sig.clone();
    let input = sig.inputs.clone();
    let out_put = sig.output.clone();
    let mut fn_args = vec![];
    let input_argv = input
        .iter()
        .skip(1)
        .map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => {
                let pat = pat_type.pat.as_ref();
                match pat {
                    syn::Pat::Ident(ident) => {
                        fn_args.push(arg.clone());
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
            if input_argv.len() > 0 {
                assert_eq!(input_argv.len(), 1);
            }
            let resource_init = if has_resource {
                let argv = input_argv[0].clone();
                quote! (
                    self.resource.call_once(|| #argv.to_owned());
                )
            } else {
                quote!()
            };

            let token = quote!(
                #(#attr)*
                #sig{
                    #resource_init
                    self.domain.read().init(#(#input_argv),*)
                }
            );
            (token, quote!())
        }
        _ => {
            let (func_inner, trampoline) = gen_trampoline(
                has_recover,
                trait_name,
                proxy_name,
                name,
                input_argv,
                fn_args,
                out_put,
            );

            let body = if no_check {
                quote! (
                    #func_inner
                )
            } else {
                quote! (
                    if !self.is_active() {
                        return Err(AlienError::DOMAINCRASH);
                    }
                    #func_inner
                )
            };

            let token = quote!(
                #(#attr)*
                #sig{
                    #body
                }
            );
            (token, trampoline)
        }
    }
}

fn gen_trampoline(
    has_recover: bool,
    trait_name: &Ident,
    proxy_name: &Ident,
    func_name: Ident,
    input_argv: Vec<Ident>,
    fn_args: Vec<FnArg>,
    out_put: ReturnType,
) -> (TokenStream, TokenStream) {
    let trampoline_ident = format!("{}_{}_trampoline", proxy_name, func_name);
    let trampoline_ident = Ident::new(&trampoline_ident, func_name.span());

    let real_ident = format!("{}_{}", proxy_name, func_name);
    let real_ident = Ident::new(&real_ident, func_name.span());

    let error_ident = format!("{}_error", real_ident);
    let error_ident = Ident::new(&error_ident, func_name.span());

    let error_ident_ptr = format!("{}_error_ptr", real_ident);
    let error_ident_ptr = Ident::new(&error_ident_ptr, func_name.span());

    if has_recover {
        let call = quote! (
            {
                let guard = self.domain.read();
                unsafe {
                    #trampoline_ident(&guard, #(#input_argv),*)
                }
            }
        );
        let asm_code = quote!(
            #[no_mangle]
            #[naked]
            #[allow(non_snake_case)]
            #[allow(undefined_naked_function_abi)]
            unsafe fn #trampoline_ident(domain:&alloc::boxed::Box<dyn #trait_name>,#(#fn_args),*) #out_put{
                core::arch::asm!(
                    "addi sp, sp, -33*8",
                    "sd x1, 1*8(sp)",
                    "sd x2, 2*8(sp)",
                    "sd x3, 3*8(sp)",
                    "sd x4, 4*8(sp)",
                    "sd x5, 5*8(sp)",
                    "sd x6, 6*8(sp)",
                    "sd x7, 7*8(sp)",
                    "sd x8, 8*8(sp)",
                    "sd x9, 9*8(sp)",
                    "sd x10, 10*8(sp)",
                    "sd x11, 11*8(sp)",
                    "sd x12, 12*8(sp)",
                    "sd x13, 13*8(sp)",
                    "sd x14, 14*8(sp)",
                    "sd x15, 15*8(sp)",
                    "sd x16, 16*8(sp)",
                    "sd x17, 17*8(sp)",
                    "sd x18, 18*8(sp)",
                    "sd x19, 19*8(sp)",
                    "sd x20, 20*8(sp)",
                    "sd x21, 21*8(sp)",
                    "sd x22, 22*8(sp)",
                    "sd x23, 23*8(sp)",
                    "sd x24, 24*8(sp)",
                    "sd x25, 25*8(sp)",
                    "sd x26, 26*8(sp)",
                    "sd x27, 27*8(sp)",
                    "sd x28, 28*8(sp)",
                    "sd x29, 29*8(sp)",
                    "sd x30, 30*8(sp)",
                    "sd x31, 31*8(sp)",
                    "call {error_ptr}",
                    "sd a0, 32*8(sp)",
                    "mv a0, sp",
                    "call register_cont",
                    //  recover caller saved registers
                    "ld ra, 1*8(sp)",
                    "ld x5, 5*8(sp)",
                    "ld x6, 6*8(sp)",
                    "ld x7, 7*8(sp)",
                    "ld x10, 10*8(sp)",
                    "ld x11, 11*8(sp)",
                    "ld x12, 12*8(sp)",
                    "ld x13, 13*8(sp)",
                    "ld x14, 14*8(sp)",
                    "ld x15, 15*8(sp)",
                    "ld x16, 16*8(sp)",
                    "ld x17, 17*8(sp)",
                    "ld x28, 28*8(sp)",
                    "ld x29, 29*8(sp)",
                    "ld x30, 30*8(sp)",
                    "ld x31, 31*8(sp)",
                    "addi sp, sp, 33*8",
                    "la gp, {real_func}",
                    "jr gp",
                    error_ptr = sym #error_ident_ptr,
                    real_func = sym #real_ident,
                    options(noreturn)
                )
            }
            #[allow(non_snake_case)]
            fn #real_ident(domain:&alloc::boxed::Box<dyn #trait_name>,#(#fn_args),*) #out_put{
                let res = domain.#func_name(#(#input_argv),*);
                continuation::pop_continuation();
                res
            }
            #[allow(non_snake_case)]
            fn #error_ident() #out_put{
                Err(AlienError::DOMAINCRASH)
            }
            #[allow(non_snake_case)]
            fn #error_ident_ptr() ->usize{
                #error_ident as usize
            }

        );
        (call, asm_code)
    } else {
        (
            quote! (
                 self.domain.read().#func_name(#(#input_argv),*)
            ),
            quote!(),
        )
    }
}
