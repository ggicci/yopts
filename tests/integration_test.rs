use ramen;

#[test]
fn test_parse_only_names() {
    const PROGRAM: &str = r#"
    version: 1.0
    program: upload
    args: [SRC, DST, -v/--verbose, -t/--threads, --protocol]
    "#;
    let optstring: Vec<String> = vec![
        "/path/to/src",
        "/path/to/dst",
        "-v",
        "--threads",
        "4",
        "--protocol",
        "scp",
    ]
    .iter()
    .map(|&x| x.to_string())
    .collect();
    let output = ramen::parse(PROGRAM, &optstring).unwrap();
    println!("{}", output);
}

#[test]
fn test_parse_boolean_flags() {
    const PROGRAM: &str = r#"
    version: 1.0
    program: upload
    args:
      - SRC
      - DST
      - name: verbose
        short: -v
        long: --verbose
        type: boolean
      - -t/--threads
      - --protocol
    "#;
    let optstring: Vec<String> = vec![
        "/path/to/src",
        "/path/to/dst",
        "-v",
        "--threads",
        "4",
        "--protocol",
        "scp",
    ]
    .iter()
    .map(|&x| x.to_string())
    .collect();
    let output = ramen::parse(PROGRAM, &optstring).unwrap();
    println!("{}", output);
}
