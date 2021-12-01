use crate::{
  constants::{
    ERROR_ATTRIBUTE_ARGS_LENGTH, ERROR_ATTRIBUTE_DATA_TYPE, ERROR_FIELD_ARRAY_DATA_TYPE, ERROR_STRUCT_NAMED_FIELD,
  },
  utils::{get_byte_size_from_integer_type, is_positive_integer, is_u8},
};
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
      .ok_or_else(|| syn::Error::new_spanned(field.into_token_stream(), ERROR_STRUCT_NAMED_FIELD))
      .unwrap();

    let attrs_ref = field.attrs.clone();

    let attribute = attrs_ref.iter().find(|attr| attr.path.is_ident("byte_me"));

    let attribute = if let Some(attribute) = attribute {
      let meta: syn::Meta = attribute.parse_args().unwrap();
      let segments = meta.path().clone().segments;

      if segments.len() == 1 {
        Some(segments.into_iter().next().unwrap().ident)
      } else {
        return Err(syn::Error::new_spanned(
          field.into_token_stream(),
          ERROR_ATTRIBUTE_ARGS_LENGTH,
        ));
      }
    } else {
      None
    };

    let field_type = field.ty.clone();

    match field_type {
      syn::Type::Array(syn::TypeArray { elem, len, .. }) => process_u8_array(elem, len, ident, field),
      syn::Type::Path(syn::TypePath { path, .. }) if is_positive_integer(&path) => {
        process_positive_integer(path, ident)
      }
      syn::Type::Path(syn::TypePath { path, .. }) if attribute.is_some() => process_enum(path, attribute, ident),
      _ => Err(syn::Error::new_spanned(
        field.into_token_stream(),
        ERROR_ATTRIBUTE_DATA_TYPE,
      )),
    }
  }
}

fn process_enum(
  path: syn::Path,
  attribute: Option<proc_macro2::Ident>,
  ident: proc_macro2::Ident,
) -> Result<ByteMeField, syn::Error> {
  let attribute_data_type = Some(path.segments.into_iter().next().unwrap().ident);
  let data_type: syn::Ident = syn::parse_quote!(#attribute);
  let size: usize = get_byte_size_from_integer_type(data_type.clone()).unwrap();
  Ok(ByteMeField {
    ident,
    size,
    data_type,
    is_array: false,
    attribute: attribute_data_type,
  })
}

fn process_positive_integer(path: syn::Path, ident: proc_macro2::Ident) -> Result<ByteMeField, syn::Error> {
  let data_type = path.segments.into_iter().next().unwrap().ident;
  let size: usize = get_byte_size_from_integer_type(data_type.clone()).unwrap();
  Ok(ByteMeField {
    ident,
    size,
    data_type,
    is_array: false,
    attribute: None,
  })
}

fn process_u8_array(
  elem: Box<syn::Type>,
  len: syn::Expr,
  ident: proc_macro2::Ident,
  field: &syn::Field,
) -> Result<ByteMeField, syn::Error> {
  let elem = elem.as_ref();
  let elem: syn::TypePath = syn::parse_quote!(#elem);
  let size: syn::LitInt = syn::parse_quote!(#len);
  let size = size.base10_parse::<usize>().unwrap();

  if is_u8(&elem.path) {
    Ok(ByteMeField {
      ident,
      size,
      data_type: syn::Ident::new("u8", proc_macro2::Span::call_site()),
      is_array: true,
      attribute: None,
    })
  } else {
    Err(syn::Error::new_spanned(
      field.into_token_stream(),
      ERROR_FIELD_ARRAY_DATA_TYPE,
    ))
  }
}
