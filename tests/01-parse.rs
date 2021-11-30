use byteme::ByteMe;
pub use num_derive::FromPrimitive;
pub use num_traits::FromPrimitive;

#[derive(Debug, FromPrimitive, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
  Unavailable = 0,
  Unauthenticated = 1,
  Authenticated = 2,
  Encrypted = 4,
}

#[derive(ByteMe, Debug, PartialEq, Eq, Clone, Copy)]
pub struct FrameOne {
  pub unused: [u8; 12],
  #[byte_me(u32)]
  pub mode: Mode,
  pub challenge: [u8; 16],
  pub salt: [u8; 16],
  pub count: u32,
  pub mbz: [u8; 12],
}

fn main() {
  let frame = FrameOne {
    unused: [0; 12],
    mode: Mode::Unauthenticated,
    challenge: [0; 16],
    salt: [0; 16],
    count: 1024,
    mbz: [0; 12],
  };

  let bytes: Vec<u8> = frame.into();
  let result: FrameOne = bytes.into();

  assert_eq!(result, frame);
}
