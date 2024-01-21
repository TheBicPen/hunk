mod parse_args;
mod test;

use console::strip_ansi_codes;
use parse_args::{parse_program_args, UTF8Strategy, Config, OutputConfig};
use std::io;

struct Chunk {
    lines: Vec<String>,
}

struct HunkDiffWithTail {
    diff: Chunk,
    context_tail: Chunk,
}

struct Hunk {
    header: String,
    context_head: Chunk,
    diffs: Vec<HunkDiffWithTail>,
}

struct FileDiff {
    file_header: Chunk,
    hunks: Vec<Hunk>,
}

struct Patch {
    patch_header: Chunk,
    files: Vec<FileDiff>,
}

enum State {
    Start,
    PatchHeader,
    FileHeader,
    HunkHead,
    HunkBodyDiff,
    HunkBodyTail,
}

fn chunk_from(line: String) -> Chunk {
    Chunk { lines: vec![line] }
}

fn chunk_empty() -> Chunk {
    Chunk { lines: Vec::new() }
}

fn print_patch<'a>(
    output_config: &OutputConfig,
    patch: &Patch,
    writer: &mut Box<dyn io::Write + 'a>
) -> std::io::Result<()> {
    match output_config {
        OutputConfig::CommitHash => {
            let commit_line = strip_ansi_codes(&patch.patch_header.lines[0]);
            let commit_hash = commit_line.strip_prefix("commit ")
                        .expect("invalid commit message line").to_string();
            write!(writer, "{}", commit_hash)?
        },
        OutputConfig::Sections(print_sections) => {
            if print_sections.patch_header {
                for line in &patch.patch_header.lines {
                    write!(writer, "{}", line)?;
                }
            }
            for file in &patch.files {
                if print_sections.file_header {
                    for line in &file.file_header.lines {
                        write!(writer, "{}", line)?;
                    }
                }
                for hunk in &file.hunks {
                    if print_sections.context {
                        write!(writer, "{}", hunk.header)?;
                        for line in &hunk.context_head.lines {
                            write!(writer, "{}", line)?;
                        }
                    }
                    for diff in &hunk.diffs {
                        if print_sections.diff {
                            for line in &diff.diff.lines {
                                write!(writer, "{}", line)?;
                            }
                        }
                        if print_sections.context {
                            for line in &diff.context_tail.lines {
                                write!(writer, "{}", line)?;
                            }
                        }
                    }
                }
            }
        },
    }
    Ok(())
}

fn process_patch<'a>(
    config: &Config,
    patch: &Patch,
    writer: &mut Box<dyn io::Write + 'a>
) -> std::io::Result<()> {
    let mut process_lines = |lines: &Vec<String>| -> std::io::Result<bool> {
        for line in lines {
            if line.contains(&config.search_string) {
                print_patch(&config.output, patch, writer)?;
                return Ok(true);
            }
        }
        Ok(false)
    };

    if config.match_on.patch_header {
        if process_lines(&patch.patch_header.lines)? {
            return Ok(());
        }
    }
    for file in &patch.files {
        if config.match_on.file_header {
            if process_lines(&file.file_header.lines)? {
                return Ok(());
            }
        }
        for hunk in &file.hunks {
            if config.match_on.context && hunk.header.contains(&config.search_string) {
                return print_patch(&config.output, patch, writer);
            }
            if config.match_on.context {
                if process_lines(&hunk.context_head.lines)? {
                    return Ok(());
                }
            }
            for diff in &hunk.diffs {
                if config.match_on.diff {
                    if process_lines(&diff.diff.lines)? {
                        return Ok(());
                    }
                }
                if config.match_on.context {
                    if process_lines(&diff.context_tail.lines)? {
                        return Ok(());
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let config = parse_program_args(&mut std::env::args());
    process_lines(
        Box::new(io::stdin().lock()), 
        Box::new(io::stdout().lock()), 
        &config)
}

fn process_lines<'a>(
        mut reader: Box<dyn io::BufRead + 'a>,
        mut writer: Box<dyn io::Write + 'a>,
        config: &Config
) -> std::io::Result<()> {
    let mut line_num = 0;
    let mut state = State::Start;
    // store only 1 patch worth of context
    let mut patch = Patch {
        patch_header: Chunk { lines: Vec::new() },
        files: Vec::new(),
    };

    loop {
        let mut line_buf: Vec<u8> = Vec::new();
        if reader.read_until(b'\n', &mut line_buf).expect("Failed to read.") == 0 {
            break;
        }
        line_num += 1;
        
        let line = match config.decode_strategy {
            UTF8Strategy::Lossy => String::from_utf8_lossy(&line_buf).into_owned(),
            UTF8Strategy::Panic => String::from_utf8(line_buf).expect(
                &format!("Invalid UTF-8 on line {line_num}")),
            UTF8Strategy::SkipLine => {
                // Choose the default value based on the state to avoid taking
                // an unnecesary state transition or adding extra rules to the
                // state machine to handle this edge case.
                let default_value = match state {
                    State::HunkHead => "+",
                    _ => " "
                }.to_string();
                String::from_utf8(line_buf).unwrap_or(default_value)
            }
        };
        
        let line_ansi_stripped = strip_ansi_codes(&line);
        let mut line_stripped = line_ansi_stripped.into_owned();
        if line_stripped.ends_with("\n") {
            line_stripped.pop();
            if line_stripped.ends_with("\r") {
                line_stripped.pop();
            }
        }

        match state {
            State::Start => {
                if line_stripped.starts_with("commit ") {
                    patch = Patch {
                        patch_header: chunk_from(line),
                        files: Vec::new(),
                    };
                    state = State::PatchHeader;
                } else {
                    panic!("Invalid patch. Expected commit message");
                }
            }
            State::PatchHeader => {
                if line_stripped.starts_with("diff --git") {
                    patch.files.push(FileDiff {
                        file_header: chunk_from(line),
                        hunks: Vec::new(),
                    });
                    state = State::FileHeader;
                } else {
                    patch.patch_header.lines.push(line);
                }
            }
            State::FileHeader => {
                let file = patch.files.last_mut().expect("Expected a file diff");
                if line_stripped.starts_with("@@") {
                    file.hunks.push(Hunk {
                        header: line,
                        context_head: chunk_empty(),
                        diffs: Vec::new(),
                    });
                    state = State::HunkHead;
                } else {
                    file.file_header.lines.push(line);
                }
            }
            State::HunkHead => {
                let file = patch.files.last_mut().expect("Expected a file diff");
                let hunk = file.hunks.last_mut().expect("Expected a hunk");
                if line_stripped.starts_with(" ") {
                    hunk.context_head.lines.push(line);
                } else if line_stripped.starts_with("+") || line_stripped.starts_with("-") {
                    hunk.diffs.push(HunkDiffWithTail {
                        diff: chunk_from(line),
                        context_tail: chunk_empty(),
                    });
                    state = State::HunkBodyDiff;
                } else {
                    panic!("Unknown state in hunk head");
                }
            }
            State::HunkBodyDiff => {
                let file = patch.files.last_mut().expect("Expected a file diff");
                let hunk = file.hunks.last_mut().expect("Expected a hunk");
                let hunk_diff = hunk.diffs.last_mut().expect("Expected a hunk diff");
                if line_stripped.starts_with("+") || line_stripped.starts_with("-") {
                    hunk_diff.diff.lines.push(line);
                } else if line_stripped.starts_with(" ")
                    || line_stripped.is_empty()
                    || line_stripped == "\\ No newline at end of file"
                {
                    hunk_diff.context_tail.lines.push(line);
                    state = State::HunkBodyTail;
                } else if line_stripped.starts_with("diff --git") {
                    patch.files.push(FileDiff {
                        file_header: chunk_from(line),
                        hunks: Vec::new(),
                    });
                    state = State::FileHeader;
                } else if line_stripped.starts_with("commit ") {
                    patch = Patch {
                        patch_header: chunk_from(line),
                        files: Vec::new(),
                    };
                    state = State::PatchHeader;
                } else {
                    panic!("Unknown state in hunk body");
                }
            }
            State::HunkBodyTail => {
                let file = patch.files.last_mut().expect("Expected a file diff");
                let hunk = file.hunks.last_mut().expect("Expected a hunk");
                let hunk_diff = hunk.diffs.last_mut().expect("Expected a hunk diff");
                if line_stripped.starts_with(" ")
                    || line_stripped.is_empty()
                    || line_stripped == "\\ No newline at end of file"
                {
                    hunk_diff.context_tail.lines.push(line);
                } else if line_stripped.starts_with("+") || line_stripped.starts_with("-") {
                    hunk_diff.diff.lines.push(line);
                    state = State::HunkBodyDiff;
                } else if line_stripped.starts_with("@@") {
                    file.hunks.push(Hunk {
                        header: line,
                        context_head: chunk_empty(),
                        diffs: Vec::new(),
                    });
                    state = State::HunkHead;
                } else if line_stripped.starts_with("diff --git") {
                    patch.files.push(FileDiff {
                        file_header: chunk_from(line),
                        hunks: Vec::new(),
                    });
                    state = State::FileHeader;
                } else if line_stripped.starts_with("commit ") {
                    process_patch(config, &patch, &mut writer)?;
                    patch = Patch {
                        patch_header: chunk_from(line),
                        files: Vec::new(),
                    };
                    state = State::PatchHeader;
                } else {
                    panic!("Unknown state in hunk tail");
                }
            }
        };
    }
    process_patch(config, &patch, &mut writer)?;
    Ok(())
}
