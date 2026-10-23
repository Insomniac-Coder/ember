# A7: a document that owns its text and keeps views of its words.
struct Document:
    text: String
    words: Array[str]

fn load(owned text: String) -> Document:
    words: Array[str] = []
    for w in text.split_whitespace():
        words.push(w)
    return Document(text=text, words=words)

fn main():
    d = load("the quick fox")
    println(d.words.len())
