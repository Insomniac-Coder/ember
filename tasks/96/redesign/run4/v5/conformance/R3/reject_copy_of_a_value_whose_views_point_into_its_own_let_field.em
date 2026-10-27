#$ test: compile-fail
#$ rules: OWN-8
#$ help: make the copy from its text: `d2 = Document(String(d.text))`
## design rules: R3, R7
## R3, R7: `copy of` is refused before a value whose views point into its own `let` field (E2116): a
## copy's words would point into the first document's text (AN-25's message).

class Document:
    let text: String
    words: Array[StrView] from text

    fn init(owns text: String):
        this.text = text
        this.words = this.text.split_whitespace().collect()

fn main():
    d = Document(String("the quick fox"))
    d2 = copy of d             #$ error[E2116]: `d.words` points into `d.text`, so a copy's words would point into `d`'s text
    println(d2.words.len())
