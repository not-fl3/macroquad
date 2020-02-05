extern crate proc_macro;
use proc_macro::{Ident, TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn macroquad_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
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

    let mut prelude: TokenStream = format!(
        "
    fn main() {{
        macroquad::Window::new(\"MACROQUAD\", amain());
    }}
    "
    )
    .parse()
    .unwrap();
    prelude.extend(modified);

    prelude
}
