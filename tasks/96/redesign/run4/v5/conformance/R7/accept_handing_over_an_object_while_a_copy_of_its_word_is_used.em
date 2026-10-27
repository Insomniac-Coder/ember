#$ test: run-pass
#$ rules: BRW-17
#$ stdout: the 3
## design rules: R7, R1
## R7: the way out E3060's message gives: `w` is a copy of your own (`String(d.words[0])`), not a view
## of `d`, so `d` may be handed over while `w` is used. The nearest program to
## `reject_handing_over_an_object_while_a_view_read_out_of_it_is_used.em` that is allowed.

class Document:
    let text: String
    words: Array[StrView] from text

    fn init(owns text: String):
        this.text = text
        this.words = this.text.split_whitespace().collect()

fn main():
    d = Document(String("the quick fox"))
    w = String(d.words[0])
    e = d
    println(w, e.words.len())
