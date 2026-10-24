# A25 when the words view a parameter (the caller's text): Rust accepts Vec<&'a str> here.
fn words_of(text: str) -> Array[str]:
    words: Array[str] = text.split_whitespace().collect()
    return words

fn main():
    t = "a b c"
    println(words_of(t).len())
