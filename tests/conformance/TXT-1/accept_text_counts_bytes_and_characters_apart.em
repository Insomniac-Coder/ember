#$ test: run-pass
#$ rules: TXT-1
#$ stdout: 6 6 5
#$ 6 5
## `[TXT-1]` — `StrView` is a read-only view of bytes known to be valid UTF-8, and `String` owns a
## buffer with the same guarantee: the length is in bytes, the characters are decoded from them.

fn main():
    t = "héllo"
    println(t.len(), t.as_bytes().len(), t.chars().count())
    kept = t.to_string()
    println(kept.len(), kept.chars().count())
