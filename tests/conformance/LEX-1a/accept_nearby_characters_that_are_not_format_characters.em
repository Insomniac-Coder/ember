#$ test: run-pass
#$ rules: LEX-1a
#$ stdout: 3
## `[LEX-1a]` - only the two ranges are refused: U+2029, U+202F, U+2065 and U+206A sit beside them and
## are not format characters of the rule.

fn main():
    # beside the range:     ⁥ ⁪
    s = " "
    println(s.len())
