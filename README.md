# hunk

A simple tool for filtering diff hunks.

## Usage

To show only hunks that contain PATTERN, run `... | hunk PATTERN` where `...`
produces a git-compatible diff.

### Examples

Show hunks from the latest 10 commits of the current git branch that contain
the string "player": `git log -p -n 10 --color | hunk player`


## Building

`cargo build`

## Testing

`cargo test`