#$ test: run-pass
#$ rules: R7
#$ stdout: 3
## R7: a view field of a class may say `from <let field>`, a `let` field of the same object whose
## contents live in their own heap block; the views stay valid for the object's whole life. AN-25's NEW
## program.

class Document:
    let text: String
    words: Array[StrView] from text

    fn init(owns text: String):
        this.text = text
        this.words = this.text.split_whitespace().collect()

fn main():
    d = Document(String("the quick fox"))
    println(d.words.len())
