use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, Token, ExprReference, parse_macro_input};

struct Args {
    assembler_expr: ExprReference,
    assembly_lines: Vec<AssemblyLine>
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let assembler_expr: ExprReference = input.parse()?;
        if assembler_expr.mutability.is_none() {
            panic!("Needs to be mutable reference!");
        }
        let _: syn::Result<Token!(,)> = input.parse();
        Ok(Self {
            assembler_expr,
            assembly_lines: vec![]
        })
    }
}

enum AssemblyLine {
    Empty,
    StaticLabel(String),
    Instruction(String, Vec<String>)
}

impl Parse for AssemblyLine {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self::Empty)
    }
}

#[proc_macro]
pub fn asm(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Args);
    quote!{ () }.into()
}