/// checks if `syn::Path` is of type `u8`
pub(crate) fn is_u8(path: &syn::Path) -> bool {
    path.clone().is_ident("u8")
}

/// checks if `syn::Path` is any of type `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
pub(crate) fn is_positive_integer(path: &syn::Path) -> bool {
    path.clone().is_ident("u8")
        || path.clone().is_ident("u16")
        || path.clone().is_ident("u32")
        || path.clone().is_ident("u64")
        || path.clone().is_ident("u128")
        || path.clone().is_ident("usize")
}

/// Returns the size of the field in bytes given the data type as a `syn::Ident`.
pub(crate) fn get_byte_size_from_integer_type(ident: syn::Ident) -> Result<usize, syn::Error> {
    match ident.to_string().as_str() {
        "u8" => Ok(1),
        "u16" => Ok(2),
        "u32" => Ok(4),
        "u64" => Ok(8),
        "u128" => Ok(16),
        "usize" => Ok(8),
        _ => Err(syn::Error::new_spanned(
            ident,
            "Unsupported type. We can only process positive integers or Enums",
        )),
    }
}
