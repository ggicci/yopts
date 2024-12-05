use ramen;

#[test]
fn test_parse_1() {
    const PROGRAM: &str = r#"
    version: 1.0
    program: upload
    args: [SRC DST -v/--verbose -t/--threads --protocol]
    "#;
    let output = ramen::parse(PROGRAM).unwrap();
    println!("{}", output);
}
