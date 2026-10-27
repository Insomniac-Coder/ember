#$ test: compile-fail
#$ rules: R7
#$ help: this.words.push(String(other_word))
## R7: a `from text` field takes only views into `text` (or text written in the program); a view of a
## parameter's text is refused (E3063). AN-25, point 6.

class Document:
    let text: String
    words: Array[StrView] from text

    fn init(owns text: String):
        this.text = text
        this.words = this.text.split_whitespace().collect()

    fn add_word(line: StrView) changes words:
        other_word = line.trim()
        this.words.push(other_word)    #$ error[E3063]: `words` holds views of `text` only, and `other_word` points into `line`; keep your own copies: write the field as `words: Array[String] = []`, in `init` write `for w in this.text.split_whitespace(): this.words.push(String(w))` in place of `this.words = this.text.split_whitespace().collect()`, and write `this.words.push(String(other_word))` in place of this line

fn main():
    d = Document(String("a b"))
    d.add_word(" c ")
    println(d.words.len())
