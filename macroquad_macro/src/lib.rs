extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn macroquad_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut main = item.to_string();

    // remove leading spaces
    main = main.trim_start().to_string();

    if main.starts_with("async") == false {
        panic!("#[macroquad_main] should be only used on async functions");
    }
    main = main["async".len()..].to_string();

    // remove spaces between "async" and "fn"
    main = main.trim_start().to_string();

    if main.starts_with("fn") == false {
        panic!("#[macroquad_main] should be only used on async functions");
    }
    main = main["fn".len()..].to_string();

    // remove spaces between "fn" and "main"
    main = main.trim_start().to_string();
    if main.starts_with("main") == false {
        panic!("#[macroquad_main] should be only used only on main");
    }

    format!(
        "
    fn main() {{
        macroquad::Window::new(\"Macroquad!\", amain());
    }}

        async fn a{}
    ",
        main
    )
    .parse()
    .unwrap()
}
