use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Expr, FnArg, Ident, ItemFn, Member, Pat,
};

struct Args {
    vars: Vec<(Ident, Expr)>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let fields = Punctuated::<syn::FieldValue, Comma>::parse_terminated(input)?;

        let mut vars = Vec::new();
        for field in fields {
            let name = match field.member {
                Member::Named(name) => name,
                Member::Unnamed(_) => {
                    panic!("only supported name field")
                }
            };
            vars.push((name, field.expr));
        }

        Ok(Args { vars })
    }
}

#[proc_macro_attribute]
pub fn smoketest(args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as ItemFn);
    let args = syn::parse_macro_input!(args as Args);
    let name = ast.sig.ident;

    if args.vars.len() > 3 {
        panic!("cannot generate macro with more than 3 arguments")
    }

    if ast.sig.variadic.is_some() {
        panic!("cannot generate smoketest with variadic functions");
    }

    let mut fnargs = Vec::new();
    for ((i, fnarg), arg) in ast.sig.inputs.iter().enumerate().zip(args.vars) {
        match fnarg {
            FnArg::Receiver(_) => panic!("cannot process receiver"),
            FnArg::Typed(t) => {
                let arg_ident = match t.pat.as_ref() {
                    Pat::Ident(i) => i.ident.clone(),
                    _ => panic!("function argument not supported"),
                };

                if arg.0.to_string() != arg_ident.to_string() {
                    panic!(
                        "function argument {}: '{}' doesn't match expected generator '{}'",
                        i, arg.0, arg_ident
                    );
                }

                let ty = t.ty.clone();
                fnargs.push((arg_ident, ty, arg.1));
            }
        }
    }

    let nb_args = fnargs.len();

    let property_body = ast.block;

    let (forall_body, ensure_body) = if nb_args == 1 {
        let fnarg = &fnargs[0];
        let arg_name = &fnarg.0;
        let body = &fnarg.2;
        let forall_body = quote! { #body };
        let ensure_body = quote! { |#arg_name| #property_body };
        (forall_body, ensure_body)
    } else {
        let product_ident = quote::format_ident!("product{}", nb_args);
        let generators = fnargs.iter().map(|x| &x.2).collect::<Vec<_>>();
        let arg_names = fnargs.iter().map(|x| &x.0).collect::<Vec<_>>();
        let tupler = quote! { | #(#arg_names),* | ( #(#arg_names),* ) };
        let forall_body = quote! {
            ::smoke::generator::#product_ident ( #(#generators),* , #tupler)
        };
        let ensure_body = quote! { | (#(#arg_names),* )| #property_body };

        (forall_body, ensure_body)
    };

    let tokens = quote! {
        #[test]
        fn #name() {
            use ::smoke::Testable;
            ::smoke::run(|ctx| ::smoke::forall(#forall_body).ensure(#ensure_body).run(ctx))
        }
    };
    TokenStream::from(tokens)
}
