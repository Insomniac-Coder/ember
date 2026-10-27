#$ test: compile-fail
#$ rules: R7, R9
#$ help: word: String
## R7: a value that holds views lives no longer than what they point into, whoever owns it; an owner made
## with `shares` is counted while the program runs, but the count keeps the holder alive, not the text
## its view points into, so the bound is still checked when compiling (E3063). Completeness check, G20.

struct Holder:
    word: StrView

fn keep(line: StrView) -> shares Holder:
    text = String(line)
    h = shares Holder(word=text.trim())
    return h    #$ error[E3063]: `h` holds a view of `text` (line 12), which ends when `keep` returns, and an owner made with `shares` does not keep `text` alive; give the holder its own text: write `word: String` in `Holder` in place of line 9 and `h = shares Holder(word=String(text.trim()))` in place of line 13

fn main():
    h = keep(" a ")
    println(h.word)
