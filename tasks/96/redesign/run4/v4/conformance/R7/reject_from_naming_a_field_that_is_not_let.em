#$ test: compile-fail
#$ rules: R7
#$ help: let text: String
## R7: `from` names a `let` field, which never changes after `init`; a field that may change is refused
## (E2118). AN-25, point 6.

class Document:
    text: String
    words: Array[StrView] from text    #$ error[E2118]: `words` points into `text`, so `text` must never change after `init`; write `let text: String`

    fn init(owns text: String):
        this.text = text
        this.words = this.text.split_whitespace().collect()

fn main():
    d = Document(String("the quick fox"))
    println(d.words.len())
