#$ test: compile-fail
#$ rules: LEX-26
# `[LEX-26]` — a string left unclosed is `E0101` (unclosed delimiter or literal), not a malformed one.

fn main():
    s = "never closed    #$ error[E0101]: unterminated string
    println(1)
