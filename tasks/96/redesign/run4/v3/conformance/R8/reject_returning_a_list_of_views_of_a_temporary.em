#$ test: compile-fail
#$ rules: R8, R7
#$ help: fn words_of(line: StrView) -> Array[String]:
## R8 (d): a list of views of a temporary may not be returned; it would outlive the lowered text.
## AN-07, point 6.

fn words_of(line: StrView) -> Array[StrView]:
    words = line.to_lower().split_whitespace().collect()
    return words    #$ error[E3063]: `words` points into the lowered copy of `line` that `line.to_lower()` made on line 8, which ends when `words_of` returns; return copies of your own: write the first line as `fn words_of(line: StrView) -> Array[String]:` and, in place of line 8, `words: Array[String] = []` then `for w in line.to_lower().split_whitespace(): words.push(String(w))`; or lower the text in the caller and return views of it: `fn words_of(text: StrView) -> Array[StrView] from text:`

fn main():
    println(words_of("A b").len())
