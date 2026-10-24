# Attack on A7 through class handles. The answer's extra checkpoint for objects is "before any call
# that runs program code"; but this function itself reads through a second handle, with no call
# between the write that frees the characters and the read.
struct Document:
    text: String
    words: Array[str]

fn load(owned text: String) -> Document:
    return Document(text=text, words=text.split_whitespace().collect())

class Holder:
    doc: Document

fn main():
    h1 = Holder(doc=load("the quick brown fox"))
    h2 = h1
    h2.doc.text = "zzz"
    println(h1.doc.words[0])
    h2.doc.words = h2.doc.text.split_whitespace().collect()
