#$ test: run-pass
#$ rules: TXT-3
#$ stdout: 3
#$ 3 true
## `[TXT-3]` — `StrView` and `String` are a pointer and a length, not null-terminated, and may
## contain `\0`.

fn main():
    s = "a\0b"
    println(s.len())
    kept = s.to_string()
    println(kept.len(), kept.as_bytes()[1] == 0)
