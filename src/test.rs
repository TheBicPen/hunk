#[cfg(test)]
mod tests {
    use crate::{parse_args::{parse_args, UTF8Strategy}, process_lines};
    use std::{fs, io::BufReader};

    #[test]
    fn test_1() {
        let file = fs::File::open("test_data/1.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            "AIPlayer".to_string(),
            UTF8Strategy::Panic).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        assert!(out_str.contains("AIPlayer"));
    }

    #[test]
    fn test_1_color() {
        let file = fs::File::open("test_data/1_color.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            "AIPlayer".to_string(),
            UTF8Strategy::Panic).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        assert!(out_str.contains("AIPlayer"));
    }

    #[test]
    fn test_unicode_chars_cjk() {
        // file may not actually contain invalid unicode
        let file = fs::File::open("test_data/unicode_chars_CJK.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            "修复安装包许可协议乱码问题".to_string(),
            UTF8Strategy::Panic).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        assert!(out_str.contains("修复安装包许可协议乱码问题"));
    }

    #[test]
    #[should_panic]
    fn test_invalid_unicode_hunk_panic() {
        let file = fs::File::open("test_data/invalid_unicode_hunk.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            "fg".to_string(),
            UTF8Strategy::Panic).unwrap();
    }

    #[test]
    fn test_invalid_unicode_hunk_skip_line() {
        let file = fs::File::open("test_data/invalid_unicode_hunk.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            "fg".to_string(),
            UTF8Strategy::SkipLine).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        assert!(!out_str.contains("fg"));
        assert!(out_str.is_empty());
    }

    #[test]
    fn test_invalid_unicode_hunk_lossy() {
        let file = fs::File::open("test_data/invalid_unicode_hunk.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            "fg".to_string(),
            UTF8Strategy::Lossy).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        assert!(out_str.contains("fg"));
    }

    #[test]
    #[should_panic]
    fn test_invalid_unicode_whole_hunk_panic() {
        let file = fs::File::open("test_data/invalid_unicode_whole_hunk.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            "Invalid".to_string(),
            UTF8Strategy::Panic).unwrap();
    }

    #[test]
    fn test_invalid_unicode_whole_hunk_skip_line() {
        let file = fs::File::open("test_data/invalid_unicode_whole_hunk.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            "Invalid".to_string(),
            UTF8Strategy::SkipLine).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        assert!(!out_str.contains("Invalid"));
        assert!(out_str.is_empty());
    }

    #[test]
    fn test_invalid_unicode_whole_hunk_lossy() {
        let file = fs::File::open("test_data/invalid_unicode_whole_hunk.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            "Invalid".to_string(),
            UTF8Strategy::Lossy).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        assert!(out_str.contains("Invalid"));
    }

    #[test]
    #[ignore = "broken"]
    fn test_empty_file_section() {
        let file = fs::File::open("test_data/empty_file_section.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            "mingw".to_string(),
            UTF8Strategy::Panic).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        assert!(out_str.contains("mingw"));
    }
    #[test]
    #[ignore = "broken"]
    fn test_empty_file_tail() {
        let file = fs::File::open("test_data/empty_file_tail.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            "mingw".to_string(),
            UTF8Strategy::Panic).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        assert!(out_str.contains("mingw"));
    }

    #[test]
    fn test_tail() {
        let file = fs::File::open("test_data/unique_tail.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            "console.log(w, l, m);".to_string(),
            UTF8Strategy::Panic).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        assert!(out_str.contains("console.log(w, l, m);"));
    }

    #[test]
    fn test_empty() {
        let file = fs::File::open("test_data/empty.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            "hello".to_string(),
            UTF8Strategy::Panic).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        assert!(out_str.is_empty());
    }

    #[test]
    fn test_only_header() {
        let file = fs::File::open("test_data/only_header.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            "hello".to_string(),
            UTF8Strategy::Panic).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        assert!(out_str.is_empty());
    }

    #[test]
    fn test_only_header_and_file_header_tail() {
        let file = fs::File::open("test_data/only_header_and_file_header_tail.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            "hi".to_string(),
            UTF8Strategy::Panic).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        assert!(out_str.is_empty());
    }

    #[test]
    fn test_parse_args() {
        let config = parse_args(&vec!["asd"]);
        assert_eq!(config.search_string, "asd");
    }

    #[test]
    #[should_panic]
    fn test_parse_extra_positional() {
        parse_args(&vec!["asd", "qwe"]);
    }

    #[test]
    #[should_panic]
    fn test_parse_no_program_name() {
        parse_args(&vec![]);
    }

    #[test]
    fn test_parse_match_fields() {
        let config = parse_args(&vec!["asd", "--match-fields", "diff,context"]);
        assert_eq!(config.search_string, "asd");
        assert_eq!(config.match_on.diff, true);
        assert_eq!(config.match_on.context, true);
        assert_eq!(config.match_on.file_header, false);
        assert_eq!(config.match_on.patch_header, false);
    }

    #[test]
    #[should_panic]
    fn test_parse_match_fields_repeat_positional_after() {
        parse_args(&vec!["asd", "--match-fields", "diff,context", "qwe"]);
    }

    #[test]
    #[should_panic]
    fn test_parse_match_on_invalid() {
        parse_args(&vec!["asd", "--match-on", "qwe"]);
    }

    #[test]
    #[should_panic]
    fn test_parse_utf8_invalid() {
        parse_args(&vec!["asd", "--invalid-utf8", "qwe"]);
    }

    #[test]
    #[should_panic]
    fn test_parse_utf8_missing() {
        parse_args(&vec!["asd", "--invalid-utf8"]);
    }

    #[test]
    fn test_parse_utf8_valid() {
        parse_args(&vec!["asd", "--invalid-utf8", "skip-line"]);
    }

    #[test]
    #[should_panic]
    fn test_parse_help() {
        parse_args(&vec!["asd", "-h"]);
    }
}
