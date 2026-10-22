#$ test: compile-fail
#$ rules: LEX-26
# `[LEX-26]` — an f-string's `{` left unclosed is `E0101` (unclosed delimiter or literal).

fn main():
    s = f"x {1 + 2"    #$ error[E0101]: unterminated `{` in an f-string
    println(s)
