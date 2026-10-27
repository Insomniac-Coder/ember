#$ test: run-pass
#$ rules: OWN-8, LT-45
#$ stdout: 3
## design rules: R3, R7
## R3, R7: the nearest program that is allowed: a second document made from a copy of the text, whose
## words point into its own text (AN-25).

class Document:
    let text: String
    words: Array[StrView] from text

    fn init(owns text: String):
        this.text = text
        this.words = this.text.split_whitespace().collect()

fn main():
    d = Document(String("the quick fox"))
    d2 = Document(String(d.text))
    println(d2.words.len())
