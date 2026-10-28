#$ test: compile-fail
#$ rules: R7, R8, R1
#$ help: w = String(d.words[0])
## R7: a view read out of a `from` field (`w = d.words[0]`) is a view of `d`, so `d` may not be handed
## over or ended while `w` is used (E3060 at the use). AN-25, rule 4.

class Document:
    let text: String
    words: Array[StrView] from text

    fn init(owns text: String):
        this.text = text
        this.words = this.text.split_whitespace().collect()

fn main():
    d = Document(String("the quick fox"))
    w = d.words[0]
    e = d
    println(w, e.words.len())    #$ error[E3060]: `e = d` (line 18) hands `d` over to `e`, and `w` (line 17) is a view of `d` used on line 19; use `w` before line 18, or keep your own copy: write `w = String(d.words[0])` in place of line 17
