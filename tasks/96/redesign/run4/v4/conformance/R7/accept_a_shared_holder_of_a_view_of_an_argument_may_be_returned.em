#$ test: run-pass
#$ rules: R7, R9, R6
#$ stdout: a
## R7: the nearest program to `reject_a_shared_holder_of_a_view_returned_past_its_text.em` that is
## allowed: the counted holder views the caller's text (R6: a result points into its view arguments),
## which outlives the call. Completeness check, G20.

struct Holder:
    word: StrView

fn keep(line: StrView) -> shares Holder:
    return shares Holder(word=line.trim())

fn main():
    h = keep(" a ")
    println(h.word)
