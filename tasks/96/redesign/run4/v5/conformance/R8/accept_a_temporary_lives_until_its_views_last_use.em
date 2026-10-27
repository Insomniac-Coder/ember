#$ test: run-pass
#$ rules: BRW-16
#$ stdout: 5
## design rules: R8
## R8 (d), row 5: a temporary lives until the last use of every view made from it; the lowered text is
## kept until `word`'s last use. AN-08's NEW program.

fn first_word_len(line: StrView) -> int:
    word = line.to_lower().trim()
    return word.len()

fn main():
    println(first_word_len("  Hello  "))
