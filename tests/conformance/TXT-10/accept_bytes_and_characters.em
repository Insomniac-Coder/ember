#$ test: run-pass
#$ rules: TXT-10, TXT-11
#$ profiles: debug, release, shipping
#$ stdout: 6 5 false
#$ 6 5 false true
# `len()` counts bytes and `char_count()` characters (Python's `len`); a
# `String` has both through its `str`.

fn main():
    name = "héllo"
    text: String = "héllo"
    println(name.len(), name.char_count(), name.is_empty())
    println(text.len(), text.char_count(), text.is_empty(), "".is_empty())
