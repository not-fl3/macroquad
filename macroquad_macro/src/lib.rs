extern crate proc_macro;
use proc_macro::{Ident, TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut modified = TokenStream::new();
    let mut source = item.into_iter();

    if let TokenTree::Ident(ident) = source.next().unwrap() {
        assert_eq!(format!("{}", ident), "async");

        modified.extend(std::iter::once(TokenTree::Ident(ident)));
    } else {
        panic!("[macroquad::main] is allowed only for async functions");
    }

    if let TokenTree::Ident(ident) = source.next().unwrap() {
        assert_eq!(format!("{}", ident), "fn");

        modified.extend(std::iter::once(TokenTree::Ident(ident)));
    } else {
        panic!("[macroquad::main] is allowed only for functions");
    }

    if let TokenTree::Ident(ident) = source.next().unwrap() {
        assert_eq!(format!("{}", ident), "main");

        modified.extend(std::iter::once(TokenTree::Ident(Ident::new(
            "amain",
            ident.span(),
        ))));
    } else {
        panic!("[macroquad::main] expecting main function");
    }
    modified.extend(source);

    let (method, ident) = match attr.into_iter().next() {
        Some(TokenTree::Ident(ident)) => ("from_config", format!("{}()", ident)),
        Some(TokenTree::Literal(literal)) => ("new", literal.to_string()),
        Some(wrong_ident) => panic!("Wrong argument: {:?}. Place function returned `Conf`", wrong_ident),
        None => panic!("No argument! Place function returned `Conf`"),
    };

    let mut prelude: TokenStream = format!(
        "
    fn main() {{
        macroquad::Window::{method}({ident}, amain());
    }}
    ", method=method, ident=ident
    )
    .parse()
    .unwrap();
    prelude.extend(modified);

    prelude
}
