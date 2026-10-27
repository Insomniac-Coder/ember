#$ test: compile-fail
#$ rules: R2
#$ help: for a view of your own write `rest = line.trim()`
## R2: `=` on a plain parameter that holds a view is refused too (E3023): it stays the text the
## caller passed (`V4/rules.md` section 3's message).

fn rest_len(line: StrView) -> int:
    line = line.trim()         #$ error[E3023]: `line` is the text the caller passed and stays that text for the whole call
    return line.len()

fn main():
    println(rest_len("  ab "))
