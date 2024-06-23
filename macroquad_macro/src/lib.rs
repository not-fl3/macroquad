extern crate proc_macro;
use proc_macro::{Ident, TokenStream, TokenTree};

use std::iter::Peekable;

fn next_group(source: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<proc_macro::Group> {
    if let Some(TokenTree::Group(_)) = source.peek() {
        let group = match source.next().unwrap() {
            TokenTree::Group(group) => group,
            _ => unreachable!("just checked with peek()!"),
        };
        Some(group)
    } else {
        None
    }
}

fn next_literal(source: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<String> {
    if let Some(TokenTree::Literal(lit)) = source.peek() {
        let mut literal = lit.to_string();

        // the only way to check that literal is string :/
        if literal.starts_with("\"") {
            literal.remove(0);
            literal.remove(literal.len() - 1);
        }
        source.next();
        return Some(literal);
    }

    return None;
}

#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut modified = TokenStream::new();
    let mut source = item.into_iter().peekable();

    let mut crate_rename = None;

    while let Some(TokenTree::Punct(punct)) = source.peek() {
        assert_eq!(format!("{}", punct), "#");

        // skip '#'
        let _ = source.next().unwrap();

        let group = next_group(&mut source);
        let mut group = group.unwrap().stream().into_iter().peekable();
        let attribute_name = format!("{}", group.next().unwrap());

        // skipping non-relevant attributes
        if attribute_name == "macroquad" {
            let group = next_group(&mut group);
            let mut group = group.unwrap().stream().into_iter().peekable();
            let config_name = format!("{}", group.next().unwrap());

            if group.peek().is_some() {
                // skip '='
                let _ = group.next();

                let config_value = Some(next_literal(&mut group).unwrap());

                if config_name == "crate_rename" {
                    crate_rename = config_value;
                }
            }
        }
    }

    let is_pub = if let TokenTree::Ident(ident) = source.peek().unwrap() {
        if ident.to_string() == "pub" {
            // skip 'pub'
            let _ = source.next().unwrap();
            true
        } else {
            false
        }
    } else {
        false
    };

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

    modified.extend(std::iter::once(source.next().unwrap()));

    let next = source.next().unwrap();
    let use_result = if let TokenTree::Punct(punct) = &next {
        format!("{}", punct) == "-" // Start of `-> Result<(), ...>`
    } else {
        false
    };
    modified.extend(std::iter::once(next));
    modified.extend(source);

    let (method, ident) = match attr.into_iter().next() {
        Some(TokenTree::Ident(ident)) => ("from_config", format!("{}()", ident)),
        Some(TokenTree::Literal(literal)) => ("new", literal.to_string()),
        Some(wrong_ident) => panic!(
            "Wrong argument: {:?}. Place function returned `Conf`",
            wrong_ident
        ),
        None => panic!("No argument! Place function returned `Conf`"),
    };

    let crate_name = crate_rename.unwrap_or_else(|| "macroquad".to_string());
    let mut prelude: TokenStream = format!(
        "
    {pub_main} fn main() {{
        {crate_name}::Window::{method}({ident}, {main});
    }}
    ",
        pub_main = if is_pub { "pub" } else { "" },
        crate_name = crate_name,
        method = method,
        ident = ident,
        main = if use_result {
            format!(
                "async {{
                if let Err(err) = amain().await {{
                    {}::logging::error!(\"Error: {{:?}}\", err);
                }}
            }}",
                crate_name
            )
        } else {
            "amain()".to_string()
        }
    )
    .parse()
    .unwrap();
    prelude.extend(modified);

    prelude
}

// Right now it is a copy-paste from "main"
// maybe it is worth it to move reuse the code from main (it would be easy -
// test is pretty much the same thing, but adding #[test] and not panicing when
// function is not called "main")
// But for now I am not really sure what exactly #[macroquad::test] should do,
// so for easier modifications - it is decoupled from #[macroquad::main]
#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut modified = TokenStream::new();
    let mut source = item.into_iter().peekable();

    while let Some(TokenTree::Punct(punct)) = source.peek() {
        assert_eq!(format!("{}", punct), "#");
        // skip '#'
        let _ = source.next().unwrap();
        let _group = next_group(&mut source);
    }

    if let TokenTree::Ident(ident) = source.next().unwrap() {
        assert_eq!(format!("{}", ident), "async");

        modified.extend(std::iter::once(TokenTree::Ident(ident)));
    } else {
        panic!("[macroquad::test] is allowed only for async functions");
    }

    if let TokenTree::Ident(ident) = source.next().unwrap() {
        assert_eq!(format!("{}", ident), "fn");

        modified.extend(std::iter::once(TokenTree::Ident(ident)));
    } else {
        panic!("[macroquad::test] is allowed only for functions");
    }

    let test_name = if let TokenTree::Ident(ident) = source.next().unwrap() {
        let test_name = format!("{}", ident);

        modified.extend(std::iter::once(TokenTree::Ident(Ident::new(
            &format!("{}_async", test_name),
            ident.span(),
        ))));
        test_name
    } else {
        panic!("[macroquad::test] expecting main function");
    };

    modified.extend(std::iter::once(source.next().unwrap()));

    modified.extend(source);

    let mut prelude: TokenStream = format!(
        "
    #[test]
    fn {test_name}() {{
        let _lock = unsafe {{
          let mutex = macroquad::test::ONCE.call_once(|| {{
            macroquad::test::MUTEX = Some(std::sync::Mutex::new(()));
          }});
          macroquad::test::MUTEX.as_mut().unwrap().lock()
        }};
        macroquad::Window::new(\"test\", {test_name}_async());
    }}
    ",
        test_name = test_name,
    )
    .parse()
    .unwrap();
    prelude.extend(modified);

    prelude
}

/// Very experimental thing for macroquad::experimantal::scene
/// Maybe will go away in future versions
#[doc(hidden)]
#[proc_macro_derive(CapabilityTrait)]
pub fn capability_trait_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut source = input.into_iter().peekable();

    while let Some(TokenTree::Punct(_)) = source.peek() {
        let _ = source.next();
        let _ = next_group(&mut source);
    }
    assert_eq!("pub", &format!("{}", source.next().unwrap()));
    assert_eq!("struct", &format!("{}", source.next().unwrap()));
    let struct_name = format!("{}", source.next().unwrap());

    let mut group = next_group(&mut source)
        .unwrap()
        .stream()
        .into_iter()
        .peekable();

    let mut trait_decl = format!("pub trait {}Trait {{", struct_name);
    let mut trait_impl = format!("impl {}Trait for NodeWith<{}> {{", struct_name, struct_name);

    fn next_str(group: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<String> {
        group.next().map(|tok| format!("{}", tok))
    }

    loop {
        // skip doc comments
        while let Some(TokenTree::Punct(_)) = group.peek() {
            let _ = group.next();
            let _ = next_group(&mut group);
        }

        let _pub = next_str(&mut group);
        if _pub.is_none() {
            break;
        }
        assert_eq!("pub", _pub.unwrap());
        let fn_name = next_str(&mut group).unwrap();
        let mut fn_res = "()".to_string();
        assert_eq!(":", &next_str(&mut group).unwrap());
        assert_eq!("fn", &next_str(&mut group).unwrap());
        let fn_args_decl = next_str(&mut group).unwrap();
        let mut fn_args_impl = String::new();

        let args = fn_args_decl.split(":").collect::<Vec<&str>>();
        for arg in &args[1..args.len() - 1] {
            fn_args_impl.push_str(&format!(", {}", arg.split(", ").last().unwrap()));
        }
        let p = next_str(&mut group);
        match p.as_deref() {
            Some("-") => {
                assert_eq!(">", next_str(&mut group).unwrap());
                fn_res = next_str(&mut group).unwrap();
                let _ = next_str(&mut group);
            }
            Some(",") => {}
            None => break,
            _ => panic!(),
        };

        trait_decl.push_str(&format!(
            "fn {} {} -> {};",
            fn_name,
            fn_args_decl.replace("node : HandleUntyped", "&self"),
            fn_res
        ));

        let args = fn_args_impl
            .replace("node : HandleUntyped", "")
            .replace("(", "")
            .replace(")", "");

        trait_impl.push_str(&format!(
            "fn {} {} -> {} {{",
            fn_name,
            fn_args_decl.replace("node : HandleUntyped", "&self"),
            fn_res
        ));
        trait_impl.push_str(&format!(
            "(self.capability.{})(self.node {})",
            fn_name, args
        ));
        trait_impl.push_str("}");
    }
    trait_decl.push_str("}");
    trait_impl.push_str("}");

    let res = format!(
        "{} 
{}",
        trait_decl, trait_impl
    );
    res.parse().unwrap()
}
