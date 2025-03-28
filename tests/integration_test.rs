use yopts;

#[test]
fn test_parse_only_names() {
    const PROGRAM: &str = r#"
    version: "1.0.0"
    program: upload
    args: [SRC, DST, -v/--verbose, -t/--threads, --protocol]
    "#;
    let optstring: Vec<String> = vec![
        "/path/to/src",
        "/path/to/dst",
        "-v",
        "true",
        "--threads",
        "8",
        "--protocol",
        "s3",
    ]
    .iter()
    .map(|&x| x.to_string())
    .collect();
    let output = yopts::parse(PROGRAM, &optstring).unwrap();

    expect_output(
        vec![
            "SRC=/path/to/src",
            "DST=/path/to/dst",
            "verbose=true",
            "threads=8",
            "protocol=s3",
        ],
        &output,
    )
}

#[test]
fn test_parse_boolean_flags() {
    const PROGRAM: &str = r#"
    version: "1.0.0"
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
    let output = yopts::parse(PROGRAM, &optstring).unwrap();

    expect_output(
        vec![
            "SRC=/path/to/src",
            "DST=/path/to/dst",
            "verbose=true",
            "threads=4",
            "protocol=scp",
        ],
        &output,
    )
}

fn expect_output(expected_lines: Vec<&str>, got_output: &str) {
    let mut sorted_expected_lines = expected_lines.clone();
    sorted_expected_lines.sort();

    let mut sorted_got_lines: Vec<&str> = got_output.lines().collect();
    sorted_got_lines.sort();

    assert_eq!(sorted_expected_lines, sorted_got_lines);
}
