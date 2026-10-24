# A7: the Python order of a constructor: the text moved in and split in the same literal.
struct Document:
    text: String
    words: Array[str]

fn load(owned text: String) -> Document:
    return Document(text=text, words=text.split_whitespace().collect())

fn main():
    d = load("the quick fox")
    println(d.words.len())
