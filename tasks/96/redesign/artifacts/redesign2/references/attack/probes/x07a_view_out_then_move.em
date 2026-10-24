# Attack on A7 (a value may view its own heap contents): a view copied OUT of the value. Attack 7 of
# the answer lets `docs.push(d)` move d (the characters stay put), so if `w` were tied only to "d's
# own limit", `docs.clear()` would free the characters `w` still points into.
struct Document:
    text: String
    words: Array[str]

fn load(owned text: String) -> Document:
    return Document(text=text, words=text.split_whitespace().collect())

fn main():
    d = load("the quick brown fox")
    w = d.words[0]
    docs: Array[Document] = []
    docs.push(d)
    docs.clear()
    println(w)
