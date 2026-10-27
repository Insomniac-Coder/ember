#$ test: run-pass
#$ rules: R8
#$ stdout: hi
## R8 (d), row 5: the way out the refusal gives: `String(word)` hands back a copy of your own. AN-09's
## NEW program.

fn clean(line: StrView) -> String:
    word = line.to_lower().trim()
    return String(word)

fn main():
    println(clean(" Hi "))
