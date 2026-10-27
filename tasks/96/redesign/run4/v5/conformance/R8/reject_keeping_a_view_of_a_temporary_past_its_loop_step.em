#$ test: compile-fail
#$ rules: LT-45, BRW-19
#$ help: all_words.push(String(w))
## design rules: R8, R7
## R8 (d): a view of a temporary may not be stored where it would outlive the temporary; the lowered
## text ends with each loop step and `all_words` lives longer. AN-07, point 6 (its message, with the
## list's own line added so that the line it gives compiles, R18).

fn main():
    lines = [String("A b"), String("C")]
    all_words: Array[StrView] = []
    for line in lines:
        for w in line.to_lower().split_whitespace():
            all_words.push(w)    #$ error[E3063]: `w` points into the lowered copy of `line`, which ends with this loop step, and `all_words` lives longer; push your own copy: write `all_words: Array[String] = []` in place of line 10 and `all_words.push(String(w))` in place of this line
    println(all_words.len())
