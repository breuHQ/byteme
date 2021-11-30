#[test]
fn tests() {
  let test = trybuild::TestCases::new();
  test.pass("tests/01-parse.rs");
}
