//! A `proc-macro` to convert a struct into Vec<u8> and back by implemeting `From` trait on the struct.
//! The conversion is Big Endian by default.
//!
//! We have made the following assumptions about the  the struct:
//!
//! - The struct must have fields.
//! - The fields are public.
//! - The fields have the following types
//!   - `u8`
//!   - `u16`
//!   - `u32`
//!   - `u64`
//!   - `u128`
//!   - `usize`
//!   - `[u8; N]`
//!   - an enum
//! - For enum, we must attach a `#[byte_me($size)]` attribute, where size is any of the positive integer types.
//! - The enum declration must have `#[derive(FromPrimitive)]` from the `num-derive` crate.
//!
//! The `num-derive` crate is required to generate the `FromPrimitive` trait for enums. Having said that, the same
//! functionality can be achieved using `num-enum` crate. It provides furthur control over the enum data types,
//! and might prove handy. here is the [discussion](https://github.com/illicitonion/num_enum/issues/61#issuecomment-955804109)
//! on the topic.
//!
//! # Example
//!
//! ```
//! use byteme::ByteMe;
//! pub use num_derive::FromPrimitive;
//!
//!
//! #[derive(Debug, FromPrimitive)]
//! pub enum Mode {
//!   Unavailable = 0,
//!   Unauthenticated = 1,
//!   Authenticated = 2,
//!   Encrypted = 4,
//! }
//!
//! #[derive(ByteMe, Debug)]
//! pub struct FrameOne {
//!   pub unused: [u8; 12],
//!   #[byte_me(u32)]
//!   pub mode: Mode,
//!   pub challenge: [u8; 16],
//!   pub salt: [u8; 16],
//!   pub count: u32,
//!   pub mbz: [u8; 12],
//! };
//!
//! let frame = FrameOne {
//!   unused: [0; 12],
//!   mode: Mode::Authenticated,
//!   challenge: [0; 16],
//!   salt: [0; 16],
//!   count: 1024,
//!   mbz: [0; 12],
//! };
//!
//! let size = FrameOne::SIZE; // Get the number of bytes in the frame
//! let bytes: Vec<u8> = frame.into(); // Converts the frame into vector of bytes
//! let frame: FrameOne = bytes.into(); // Converts the bytes back to frame
//! ```

mod models;
mod utils;
use crate::models::{ByteMeField, ByteMeStruct};

/// Macro to provide `from()` & `into()` implementations for a struct with either `enums` and positive integer field.
#[proc_macro_derive(ByteMe, attributes(byte_me))]
pub fn derive(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let strukt = syn::parse_macro_input!(tokens as ByteMeStruct);

    let fn_lines_to_bytes = strukt
        .fields
        .iter()
        .clone()
        .map(|field| to_bytes_fn_factory(field));

    let count = core::cell::Cell::new(0_usize);
    let fn_line_from_bytes = strukt
        .fields
        .iter()
        .clone()
        .map(|field| from_bytes_fn_factory(field, &count));

    let name = &strukt.ident;
    let size: usize = strukt
        .fields
        .clone()
        .iter()
        .clone()
        .map(|field| field.size)
        .sum();
    let fields = strukt
        .fields
        .iter()
        .clone()
        .map(|field| get_field_name(field));
    let processed = quote::quote! {
      impl #name {
        /// Size of the struct in bytes
        pub const SIZE: usize = #size;

        /// Gets the delimiter as a vector of bytes.
        pub fn get_delimiter(self) -> Vec<u8> {
          (Self::SIZE as u16).to_be_bytes().to_vec()
        }
      }

      impl From<Vec<u8>> for #name {
        fn from(bytes: Vec<u8>) -> Self {
          use num_traits::FromPrimitive;
          // Converting `[u8]` to fields based on their length.
          #(#fn_line_from_bytes)*
          // Returning the struct.
          Self {
            #(#fields)*
          }
        }
      }

      impl From<#name> for Vec<u8> {
       fn from(d: #name) -> Self {
          let mut bytes = Vec::new();
          #(#fn_lines_to_bytes)*
          bytes
       }
      }
    };

    processed.into()
}

/// Creates a line for `From<Struct> for Vec<u8>` function for a single field depending on the data_type
fn to_bytes_fn_factory(field: &ByteMeField) -> proc_macro2::TokenStream {
    let name = &field.ident;
    let data_type = &field.data_type;

    if field.is_array {
        quote::quote! {
          bytes.extend_from_slice(&(d.#name));
        }
    } else {
        quote::quote! {
          bytes.extend_from_slice(&(d.#name as #data_type).to_be_bytes().to_vec());
        }
    }
}

/// Creates a line for `From<Vec<u8>> for Struct` function for a single field depending on the data_type
fn from_bytes_fn_factory(
    field: &ByteMeField,
    count: &core::cell::Cell<usize>,
) -> proc_macro2::TokenStream {
    let name = &field.ident;
    let size = &field.size;
    let data_type = &field.data_type;

    let start = count.get();
    let end = start + field.size;
    count.set(end);

    // The first line is the same for all data types
    let lines = quote::quote! {
      let #name: [u8; #size] = bytes[#start .. #end].try_into().unwrap();
    };

    // If the field is not a `[u8]` and if it doesn't have an atrribute, we can safely assume it is a positive integer.
    // In this case, we would have to return the value as the type on the field.
    let lines = if !field.is_array && field.attribute.is_none() {
        quote::quote! {
          #lines
          let #name = #data_type::from_be_bytes(#name);
        }
    } else {
        quote::quote! {
          #lines
        }
    };

    // If the field is not an array and has an attribute byteme, we can safely assume the field is an enum.
    let lines = if !field.is_array && field.attribute.is_some() {
        let enum_name = &field.attribute.clone();
        let enum_data_type = data_type.to_string();
        let from_enum_data_type = syn::Ident::new(
            format!("from_{}", enum_data_type).as_str(),
            proc_macro2::Span::call_site(),
        );
        quote::quote! {
          #lines
          let #name = #data_type::from_be_bytes(#name);
          let #name = #enum_name::#from_enum_data_type(#name).unwrap();
        }
    } else {
        quote::quote! {
          #lines
        }
    };

    lines
}

/// Given `ByteMeField`, returns a TokenStream with only name of the field
fn get_field_name(field: &ByteMeField) -> proc_macro2::TokenStream {
    let name = field.ident.clone();
    quote::quote! {#name,}
}
