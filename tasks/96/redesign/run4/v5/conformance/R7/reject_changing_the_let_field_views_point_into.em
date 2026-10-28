#$ test: compile-fail
#$ rules: CLS-9
#$ help: d2 = Document(String("the slow fox"))
## design rules: R7
## R7: the `let` field that views point into is never changed after `init` (E2104), so the views need no
## tracking. AN-25, point 6.

class Document:
    let text: String
    words: Array[StrView] from text

    fn init(owns text: String):
        this.text = text
        this.words = this.text.split_whitespace().collect()

    fn shout():
        this.text += "!"    #$ error[E2104]: `text` is `let`, because `words` points into it: it is set in `init` and never changed after; for other text, make a new document: `d2 = Document(String("the slow fox"))`

fn main():
    d = Document(String("the quick fox"))
    d.shout()
