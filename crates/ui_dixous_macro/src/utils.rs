pub fn struct_has_field(ast: &syn::ItemStruct, field: &str) -> bool {
    let mut has_field = false;
    for i in &ast.fields {
        if let Some(id) = &i.ident
            && id.to_string() == field
        {
            has_field = true;
            break;
        }
    }
    has_field
}

pub fn get_ident_from_type(ty: &syn::Type) -> Option<&syn::Ident> {
    if let syn::Type::Path(syn::TypePath { qself: None, path }) = ty
        && let Some(last_segment) = path.segments.last()
        && matches!(last_segment.arguments, syn::PathArguments::None)
    {
        return Some(&last_segment.ident);
    }

    None
}
