#$ test: run-pass
#$ rules: LEX-1a, LEX-26
#$ stdout: 3
#$ 4
## `[LEX-1a]` - the one way to put a bidirectional format character in a program is the escape of
## `[LEX-26]`: `"\u{202E}"` is one character of three UTF-8 bytes, and the file holds none.

fn main():
    s = "\u{202E}"
    println(s.len())
    println("\u{2066}x".len())
