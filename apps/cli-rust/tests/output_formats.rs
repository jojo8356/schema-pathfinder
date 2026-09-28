//! Integration tests for output-format parsing.

use schema_pathfinder::pathfinder_core::{format_name, parse_format, SUPPORTED_FORMATS};

#[test]
fn supported_formats_are_exactly_the_five_expected() {
    assert_eq!(SUPPORTED_FORMATS.len(), 5);
    for expected in ["text", "equation", "json", "sql", "mermaid"] {
        assert!(SUPPORTED_FORMATS.contains(&expected), "missing {}", expected);
    }
}

#[test]
fn every_supported_format_round_trips() {
    for name in SUPPORTED_FORMATS {
        let format = parse_format(name).unwrap_or_else(|_| panic!("{} should parse", name));
        assert_eq!(format_name(format), name);
    }
}

#[test]
fn unknown_format_is_rejected() {
    let error = parse_format("yaml").expect_err("should fail");

    assert_eq!(error.code, "UNSUPPORTED_FORMAT");
    assert!(error.message.contains("yaml"));
}
