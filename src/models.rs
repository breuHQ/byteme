use crate::utils::{get_byte_size_from_integer_type, is_positive_integer, is_u8};
use quote::ToTokens;

/// Summary of the proc_macro::TokenStream
#[derive(Debug, Clone)]
pub struct ByteMeStruct {
  pub ident: syn::Ident,
  pub fields: Vec<ByteMeField>,
}

/// Implements parser for ByteMeStruct
impl syn::parse::Parse for ByteMeStruct {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let strukt = input.parse::<syn::ItemStruct>()?;
    let ident = strukt.ident.clone();
    let mut fields: Vec<ByteMeField> = Vec::new();
    for field in strukt.fields {
      let field = ByteMeField::try_from(&field)?;
      fields.push(field);
    }
    Ok(Self { ident, fields })
  }
}

/// Represents a single field in the struct
#[derive(Debug, Clone)]
pub struct ByteMeField {
  /// Represents the name of the fields
  pub ident: syn::Ident,
  /// Represents the number of bytes that the field takes up.
  pub size: usize,
  /// Represents the positive integer type of the field.
  pub data_type: syn::Ident,
  /// Represents if the field is an array
  pub is_array: bool,
  /// Name of the enum for which the `#[byte_me($size)]` attribute is attached.
  pub attribute: Option<syn::Ident>,
}

/// Implements TryFrom for ByteMeField
impl TryFrom<&syn::Field> for ByteMeField {
  type Error = syn::Error;

  fn try_from(field: &syn::Field) -> Result<Self, Self::Error> {
    let ident = field
      .ident
      .clone()
      .ok_or_else(|| {
        syn::Error::new_spanned(
          field.into_token_stream(),
          "`ByteMe` only works for a struct with a named field",
        )
      })
      .unwrap();

    let attrs_ref = field.attrs.clone();

    let attribute = attrs_ref.iter().find(|attr| attr.path.is_ident("byte_me"));

    let attribute = match attribute {
      Some(attribute) => {
        let meta: syn::Meta = attribute.parse_args().unwrap();
        let segments = meta.path().clone().segments;
        if segments.len() != 1 {
          return Err(syn::Error::new_spanned(
            field.into_token_stream(),
            "`byte_me` attribute can only have one argument",
          ));
        } else {
          Some(segments.into_iter().next().unwrap().ident)
        }
      }
      None => None,
    };

    let field_type = field.ty.clone();

    match field_type {
        // Check if have an array and the type is any of the unsigned integer.
        syn::Type::Array(syn::TypeArray {elem, len, ..}) => {
          let elem = elem.as_ref();
          let elem: syn::TypePath = syn::parse_quote!(#elem);
          let size: syn::LitInt = syn::parse_quote!(#len);
          let size = size.base10_parse::<usize>().unwrap();
          // Currently we only support u8 arrays
          if is_u8(&elem.path) {
            Ok(Self {
              ident,
              size,
              data_type: syn::Ident::new("u8", proc_macro2::Span::call_site()),
              is_array: true,
              attribute: None,
            })
          } else {
            Err(syn::Error::new_spanned(field.into_token_stream(), "`u8` is the only supported type for arrays"))
          }
        },
        // Check if the type is any of the unsigned integer.
        syn::Type::Path(syn::TypePath {path,..}) if is_positive_integer(&path) => {
          let data_type = path.segments.into_iter().next().unwrap().ident;
          let size: usize = get_byte_size_from_integer_type(data_type.clone()).unwrap();
          Ok(Self {
            ident,
            size,
            data_type,
            is_array: false,
            attribute: None,
          })
        },
        // Check for the custom field type (can be a struct or enum) with a `#[byte_me($type)]`
        // where type can only be a positive integer. Right now we are only assuming that it is an enum
        //
        // TODO: Add support for custom structs
        syn::Type::Path(syn::TypePath {path, ..}) if attribute.is_some() => {
          let attribute_data_type = Some(path.segments.into_iter().next().unwrap().ident);
          let data_type: syn::Ident = syn::parse_quote!(#attribute);
          let size: usize = get_byte_size_from_integer_type(data_type.clone()).unwrap();
          Ok(Self {
            ident,
            size,
            data_type,
            is_array: false,
            attribute: attribute_data_type,
          })
        },
        // Raise Error if the conditions are not met.
        _ => Err(syn::Error::new_spanned(field.into_token_stream(), "Unsupported field. Try adding `byte_me($type)` attribute to the field where $type is any of the `u8`, `u16`, `u32`, `u64`, `u128` or `usize`."))
    }
  }
}
