#$ test: run-pass
#$ rules: R7
#$ stdout: 4 !
## R7: a `from text` field takes views into `text` or text written in the program (AN-25, rule 3); the
## nearest program to `reject_storing_a_view_of_other_text_in_a_from_field.em` that is allowed.

class Document:
    let text: String
    words: Array[StrView] from text

    fn init(owns text: String):
        this.text = text
        this.words = this.text.split_whitespace().collect()

    fn add_mark():
        this.words.push("!")

fn main():
    d = Document(String("the quick fox"))
    d.add_mark()
    println(d.words.len(), d.words[3])
