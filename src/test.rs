#[cfg(test)]
mod tests {
    use crate::{parse_args::{parse_args, UTF8Strategy, Config, PatchSections, OutputConfig}, process_lines};
    use std::{fs, io::BufReader};

    const PATCH_SECTIONS_ALL: PatchSections = PatchSections {
        context: true,
        diff: true,
        file_header: true,
        patch_header: true
    };

    const PATCH_SECTIONS_NONE: PatchSections = PatchSections {
        context: false,
        diff: false,
        file_header: false,
        patch_header: false
    };

    fn expect_err<_T, E: std::error::Error>(result: Result<_T, E>) {
        println!("{}", result.err().expect("Expected an error"));
    }

    #[test]
    fn test_1() {
        let file = fs::File::open("test_data/1.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "AIPlayer".to_string();
        config.match_on.diff = true;
        let mut output_sections = PATCH_SECTIONS_NONE;
        output_sections.diff = true;
        config.output = OutputConfig::Sections(output_sections);
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        println!("{}", out_str);
        assert!(out_str.contains(&config.search_string));
    }

    #[test]
    fn test_1_color() {
        let file = fs::File::open("test_data/1_color.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "AIPlayer".to_string();
        config.match_on.diff = true;
        let mut output_sections = PATCH_SECTIONS_NONE;
        output_sections.diff = true;
        config.output = OutputConfig::Sections(output_sections);
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        println!("{}", out_str);
        assert!(out_str.contains(&config.search_string));
    }

    #[test]
    fn test_1_commit_hash() {
        let file = fs::File::open("test_data/1_color.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "AIPlayer".to_string();
        config.match_on.diff = true;
        config.output = OutputConfig::CommitHash;
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        println!("{}", out_str);
        let out_lines: Vec<&str> = out_str.split('\n').collect();
        assert!(out_lines.len() == 4);
        assert!(out_lines[0] == "bcd581d22a277d2f7e8766219f96412f516418af");
        assert!(out_lines[1] == "39512adde34a5ece411a7ef67a363fa33a333f45");
        assert!(out_lines[2] == "a9b7171d2eb0164592e20e39d9f126412a44964f");
        assert!(out_lines[3] == "");

    }

    #[test]
    fn test_unicode_chars_cjk() {
        // file may not actually contain invalid unicode
        let file = fs::File::open("test_data/unicode_chars_CJK.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "修复安装包许可协议乱码问题".to_string();
        config.match_on.diff = true;
        let mut output_sections = PATCH_SECTIONS_NONE;
        output_sections.diff = true;
        config.output = OutputConfig::Sections(output_sections);
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        println!("{}", out_str);
        assert!(out_str.contains(&config.search_string));
    }

    #[test]
    fn test_invalid_unicode_hunk_panic() {
        let file = fs::File::open("test_data/invalid_unicode_hunk.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "fg".to_string();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).err().expect("");
    }

    #[test]
    fn test_invalid_unicode_hunk_skip_line() {
        let file = fs::File::open("test_data/invalid_unicode_hunk.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "fg".to_string();
        config.decode_strategy = UTF8Strategy::SkipLine;
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        println!("{}", out_str);
        assert!(!out_str.contains(&config.search_string));
        assert!(out_str.is_empty());
    }

    #[test]
    fn test_invalid_unicode_hunk_lossy() {
        let file = fs::File::open("test_data/invalid_unicode_hunk.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "fg".to_string();
        config.decode_strategy = UTF8Strategy::Lossy;
        config.match_on.diff = true;
        let mut output_sections = PATCH_SECTIONS_NONE;
        output_sections.diff = true;
        config.output = OutputConfig::Sections(output_sections);
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        println!("{}", out_str);
        assert!(out_str.contains(&config.search_string));
    }

    #[test]
    fn test_invalid_unicode_whole_hunk_panic() {
        let file = fs::File::open("test_data/invalid_unicode_whole_hunk.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "Invalid".to_string();
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).err().expect("");
    }

    #[test]
    fn test_invalid_unicode_whole_hunk_skip_line() {
        let file = fs::File::open("test_data/invalid_unicode_whole_hunk.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "Invalid".to_string();
        config.decode_strategy = UTF8Strategy::SkipLine;
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        println!("{}", out_str);
        assert!(!out_str.contains(&config.search_string));
        assert!(out_str.is_empty());
    }

    #[test]
    fn test_invalid_unicode_whole_hunk_lossy() {
        let file = fs::File::open("test_data/invalid_unicode_whole_hunk.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "Invalid".to_string();
        config.decode_strategy = UTF8Strategy::Lossy;
        config.match_on.diff = true;
        let mut output_sections = PATCH_SECTIONS_NONE;
        output_sections.diff = true;
        config.output = OutputConfig::Sections(output_sections);
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        println!("{}", out_str);
        assert!(out_str.contains(&config.search_string));
    }

    #[test]
    fn test_empty_file_section() {
        let file = fs::File::open("test_data/empty_file_section.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "mingw".to_string();
        config.match_on.patch_header = true;
        let mut output_sections = PATCH_SECTIONS_NONE;
        output_sections.patch_header = true;
        config.output = OutputConfig::Sections(output_sections);
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        println!("{}", out_str);
        assert!(out_str.contains(&config.search_string));
    }
    #[test]
    fn test_empty_file_tail() {
        let file = fs::File::open("test_data/empty_file_tail.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "mingw".to_string();
        config.match_on.patch_header = true;
        let mut output_sections = PATCH_SECTIONS_NONE;
        output_sections.patch_header = true;
        config.output = OutputConfig::Sections(output_sections);
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        println!("{}", out_str);
        assert!(out_str.contains(&config.search_string));
    }

    #[test]
    fn test_tail() {
        let file = fs::File::open("test_data/unique_tail.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "console.log(w, l, m);".to_string();
        config.match_on.diff = true;
        let mut output_sections = PATCH_SECTIONS_NONE;
        output_sections.diff = true;
        config.output = OutputConfig::Sections(output_sections);
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        println!("{}", out_str);
        assert!(out_str.contains(&config.search_string));
    }

    #[test]
    fn test_empty() {
        let file = fs::File::open("test_data/empty.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "hello".to_string();
        config.match_on = PATCH_SECTIONS_ALL;
        config.output = OutputConfig::Sections(PATCH_SECTIONS_ALL);
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        println!("{}", out_str);
        assert!(out_str.is_empty());
    }

    #[test]
    fn test_only_header() {
        let file = fs::File::open("test_data/only_header.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "hello".to_string();
        config.match_on.patch_header = true;
        let mut output_sections = PATCH_SECTIONS_NONE;
        output_sections.patch_header = true;
        config.output = OutputConfig::Sections(output_sections);
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        println!("{}", out_str);
        assert!(out_str.is_empty());
    }

    #[test]
    fn test_only_header_and_file_header_tail() {
        let file = fs::File::open("test_data/only_header_and_file_header_tail.diff").unwrap();
        let mut out_vec: Vec<u8> = Vec::new();
        let mut config = Config::default();
        config.search_string = "hi".to_string();
        config.match_on.diff = true;
        let mut output_sections = PATCH_SECTIONS_NONE;
        output_sections.diff = true;
        config.output = OutputConfig::Sections(output_sections);
        process_lines(
            Box::new(BufReader::new(file)),
            Box::new(&mut out_vec),
            &config
        ).unwrap();
        let out_str = String::from_utf8(out_vec).unwrap();
        println!("{}", out_str);
        assert!(out_str.is_empty());
    }

    #[test]
    fn test_parse_args() {
        let config = parse_args(&vec!["asd"]).unwrap();
        assert_eq!(config.search_string, "asd");
    }

    #[test]
    fn test_parse_extra_positional() {
        expect_err(parse_args(&vec!["asd", "qwe"]));
    }

    #[test]
    fn test_parse_extra_positional_explicit() {
        expect_err(parse_args(&vec!["asd", "--", "qwe"]));
    }

    #[test]
    fn test_parse_trailing_explicit() {
        expect_err(parse_args(&vec!["asd", "--"]));
    }

    #[test]
    fn test_parse_explicit() {
        let config = parse_args(&vec!["--", "asd"]).unwrap();
        assert!(config.search_string == "asd");
    }

    #[test]
    fn test_parse_explicit_flag_like() {
        let config = parse_args(&vec!["--", "-h"]).unwrap();
        assert!(config.search_string == "-h");
    }
    
    #[test]
    fn test_parse_explicit_duplicate_flag_like() {
        let config = parse_args(&vec!["--match-fields", "diff", "--", "-h"]).unwrap();
        assert!(config.search_string == "-h");
    }
    
    #[test]
    fn test_parse_explicit_duplicate_flag_like_with_arg() {
        let config = parse_args(&vec!["--match-fields", "diff", "--", "--match-fields"]).unwrap();
        assert!(config.search_string == "--match-fields");
    }

    #[test]
    fn test_parse_no_program_name() {
        expect_err(parse_args(&vec![]));
    }

    #[test]
    fn test_parse_match_fields() {
        let config = parse_args(&vec!["asd", "--match-fields", "diff,context"]).unwrap();
        assert_eq!(config.search_string, "asd");
        assert_eq!(config.match_on.diff, true);
        assert_eq!(config.match_on.context, true);
        assert_eq!(config.match_on.file_header, false);
        assert_eq!(config.match_on.patch_header, false);
    }

    #[test]
    fn test_parse_match_fields_repeat_positional_after() {
        expect_err(parse_args(&vec!["asd", "--match-fields", "diff,context", "qwe"]));
    }

    #[test]
    fn test_parse_match_on_invalid() {
        expect_err(parse_args(&vec!["asd", "--match-on", "qwe"]));
    }

    #[test]
    fn test_parse_print_fields_commit() {
        expect_err(parse_args(&vec!["asd", "--print-fields", "diff,context", "--print-commits"]));
    }

    #[test]
    fn test_parse_commit_print_fields() {
        expect_err(parse_args(&vec!["asd", "--print-commits", "--print-fields", "diff,context"]));
    }

    #[test]
    fn test_parse_utf8_invalid() {
        expect_err(parse_args(&vec!["asd", "--invalid-utf8", "qwe"]));
    }

    #[test]
    fn test_parse_utf8_missing() {
        expect_err(parse_args(&vec!["asd", "--invalid-utf8"]));
    }

    #[test]
    fn test_parse_utf8_valid() {
        let config = parse_args(&vec!["asd", "--invalid-utf8", "skip-line"]).unwrap();
        assert_eq!(config.search_string, "asd");
        assert_eq!(config.decode_strategy, UTF8Strategy::SkipLine);
    }

    #[test]
    fn test_parse_help() {
        expect_err(parse_args(&vec!["asd", "-h"]));
    }
}
