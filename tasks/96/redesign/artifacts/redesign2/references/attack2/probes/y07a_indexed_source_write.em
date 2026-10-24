# Attack on A7 (answers report 5.3): the viewed field is written through an index. Rule 2 makes
# `docs[i].words` unusable, but `docs[j]` with j == i at run time is a different expression.
# Today this is refused earlier (views may not be stored in an Array, E3063); the proposal lifts that.
struct Document:
    text: String
    words: Array[str]

fn load(owned text: String) -> Document:
    return Document(text=text, words=text.split_whitespace().collect())

fn edit(mut docs: Array[Document], i: int, j: int):
    docs[i].text.push_str(" and more words")
    println(docs[j].words[0])
    docs[i].words = docs[i].text.split_whitespace().collect()

fn main():
    docs: Array[Document] = []
    docs.push(load("the quick brown fox"))
    edit(docs, 0, 0)
