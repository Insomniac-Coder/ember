#$ test: run-pass
#$ rules: R8, R7
#$ stdout: 3
## R8 (d): a temporary lives until the last use of every view made from it, a list of views included;
## R7 bounds the list `words` owns by the lowered text. AN-07's NEW program.

fn count_words(line: StrView) -> int:
    words = line.to_lower().split_whitespace().collect()
    return words.len()

fn main():
    println(count_words("The quick fox"))
