use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use syn::{parse_macro_input, parse_file};

mod attrs;
use attrs::Attrs;

/// 遍历 brick 源码，收集 struct 是否带 `sub` 字段、枚举各变体信息。
struct Field {
    name: String,
    r#type: String,
    has_id: bool,
}

enum CompInfo {
    Struct { has_sub: bool },
    Enum { fields: Vec<Field> },
}

fn walk(ast: &syn::File) -> HashMap<String, CompInfo> {
    let mut map = HashMap::new();
    for item in &ast.items {
        match item {
            syn::Item::Struct(s) => {
                let has_sub = s
                    .fields
                    .iter()
                    .any(|f| f.ident.as_ref().is_some_and(|i| i == "sub"));
                map.insert(s.ident.to_string(), CompInfo::Struct { has_sub });
            }
            syn::Item::Enum(e) => {
                let fields = e
                    .variants
                    .iter()
                    .map(|v| {
                        // 取第一个 field 的类型名
                        let ty = v
                            .fields
                            .iter()
                            .find_map(|f| {
                                if let syn::Type::Path(p) = &f.ty
                                    && let Some(last) = p.path.segments.last()
                                {
                                    Some(last.ident.to_string())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        // 解析 #[render_brick(has_id = "true")]
                        let has_id = v.attrs.iter().any(|a| {
                            a.path().is_ident("render_brick")
                                && a.parse_args_with(|input: syn::parse::ParseStream| {
                                    let mut has = false;
                                    while !input.is_empty() {
                                        let k: syn::Ident = input.parse()?;
                                        let _: syn::Token![=] = input.parse()?;
                                        let val: syn::LitStr = input.parse()?;
                                        if k == "has_id" && val.value() == "true" {
                                            has = true;
                                        }
                                        let _: Option<syn::Token![,]> = input.parse()?;
                                    }
                                    Ok(has)
                                })
                                .unwrap_or(false)
                        });
                        Field {
                            name: v.ident.to_string(),
                            r#type: ty,
                            has_id,
                        }
                    })
                    .collect();
                map.insert(e.ident.to_string(), CompInfo::Enum { fields });
            }
            _ => {}
        }
    }
    map
}

#[proc_macro]
pub fn gen_dispatch(input: TokenStream) -> TokenStream {
    // 解析: file = "..", entry = "Brick", object = "brick"
    let cfg = parse_macro_input!(input as Attrs);
    let file = cfg
        .get("file")
        .expect("must provide file")
        .to_owned();
    let entry = cfg.get("entry").expect("must provide entry").to_owned();
    let object = cfg.get("object").expect("must provide object").to_owned();

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&file);
    let path_str = path.to_string_lossy().into_owned();
    let txt = read_to_string(&path).unwrap_or_else(|e| panic!("read {file} failed: {e}"));
    let ast = parse_file(&txt).unwrap_or_else(|e| panic!("parse {file} failed: {e}"));
    let info = walk(&ast);

    let entry = Ident::new(&entry, Span::call_site());
    let object = Ident::new(&object, Span::call_site());

    let CompInfo::Enum { fields } = info
        .get(&entry.to_string())
        .unwrap_or_else(|| panic!("no enum {entry}"))
    else {
        panic!("not an enum");
    };

    let mut arms = Vec::new();
    for f in fields {
        let var = Ident::new(&f.name, Span::call_site());
        let comp = Ident::new(&format!("{}_", f.name), Span::call_site());
        let has_sub = info
            .get(&f.r#type)
            .map(|i| matches!(i, CompInfo::Struct { has_sub: true }))
            .unwrap_or(false);

        let call = if f.has_id {
            // 有 id 的组件：生成回退 id 计数器，传给组件
            let counter = Ident::new(
                &format!("ID_{}", f.name.to_uppercase()),
                Span::call_site(),
            );
            let tag = format!("{}-{{}}", f.name);
            quote! {
                {
                    static #counter: std::sync::LazyLock<std::sync::Mutex<u32>> =
                        std::sync::LazyLock::new(|| std::sync::Mutex::new(0));
                    let id = c.id.clone().unwrap_or_else(|| {
                        let mut tc = #counter.lock().unwrap();
                        *tc += 1;
                        format!(#tag, *tc)
                    });
                    crate::components::#comp(c.clone(), ctx, id)
                }
            }
        } else {
            quote! { crate::components::#comp(c.clone(), ctx) }
        };

        // 有 sub 的容器由组件内部递归 dispatch，宏只负责分发
        let _ = has_sub;

        arms.push(quote! {
            #entry::#var(c) => #call
        });
    }

    let out = quote! {
        // HACK: 让 rustc 追踪 brick 源码变化
        const _: &[u8] = include_bytes!(#path_str);

        pub fn dispatch(#object: &#entry, ctx: &crate::Ctx) -> leptos::prelude::AnyView {
            match #object {
                #(#arms),*
            }
        }
    };

    out.into()
}