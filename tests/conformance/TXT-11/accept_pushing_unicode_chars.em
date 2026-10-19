#$ test: run-pass
#$ rules: TXT-11
#$ profiles: debug, development, release
#$ stdout: Aéह😀Z
#$ stdout: 11
# A character appends its complete UTF-8 encoding to a String.
fn main():
    s: String = ""
    s.push('A')
    s.push('é')
    s.push('ह')
    s.push('😀')
    s.push_str("Z")
    println(s)
    println(s.len())
