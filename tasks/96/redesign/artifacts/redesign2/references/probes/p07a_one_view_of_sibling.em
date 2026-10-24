# A7 without the container question: one view field borrowing a sibling String's heap buffer.
struct Doc:
    text: String
    first: str

fn load(owned text: String) -> Doc:
    first = text[0..3]
    return Doc(text=text, first=first)

fn main():
    d = load("the quick fox")
    println(d.first)
