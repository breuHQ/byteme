# byteme

Macro to provide `from()` & `into()` implementations for a struct with either `enums` and positive integer field.

We have made the following assumptions about the fields of the struct:

- The fields are public.
- The fields will have the following types
  - `u8`
  - `u16`
  - `u32`
  - `u64`
  - `u128`
  - `usize`
  - `[u8]`
  - an enum
- For enum, we must attach a `#[byte_me($size)]` attribute, where size is any of the positive integer types.
- The enum declration must `#[derive(FromPrimitive)]` from the `num-derive` crate.

The `num-derive` crate is required to generate the `FromPrimitive` trait for enums. Having said that, the same
functionality can be achieved using `num-enum` crate. It provides furthur control over the enum data types,
and might prove handy. here is the [discussion](https://github.com/illicitonion/num_enum/issues/61#issuecomment-955804109)
on the topic.

## Example

```rust
use byteme::ByteMe;
pub use num_derive::FromPrimitive;


#[derive(Debug, FromPrimitive)]
pub enum Mode {
  Unavailable = 0,
  Unauthenticated = 1,
  Authenticated = 2,
  Encrypted = 4,
}

#[derive(ByteMe, Debug)]
pub struct FrameOne {
  pub unused: [u8; 12],
  #[byte_me(u32)]
  pub mode: Mode,
  pub challenge: [u8; 16],
  pub salt: [u8; 16],
  pub count: u32,
  pub mbz: [u8; 12],
};

let frame = FrameOne {
  unused: [0; 12],
  mode: Mode::Authenticated,
  challenge: [0; 16],
  salt: [0; 16],
  count: 1024,
  mbz: [0; 12],
};

let size = FrameOne::SIZE; // Get the number of bytes in the frame
let bytes: Vec<u8> = frame.into(); // Converts the frame into vector of bytes
let frame: FrameOne = bytes.into(); // Converts the bytes back to frame
```
