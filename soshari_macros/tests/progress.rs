#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-simple-proc.rs");
    // t.pass("tests/02-parser.rs");
    // t.pass("tests/03-missing-positional.rs");
    // t.pass("tests/04-derive-enum.rs");
    // t.pass("tests/05-verify.rs");
    // t.pass("tests/06-parsable-opt.rs");
}
