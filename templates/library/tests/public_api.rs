use std::borrow::Cow;

#[test]
fn stdlib_integration_example() {
    let input: Cow<'_, str> = "hello".into();
    assert_eq!(input.as_ref(), "hello");
}
