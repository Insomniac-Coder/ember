# A25: the same inside one function: drop the text, then read a word.
fn words_of(text: str) -> Array[str]:
    words: Array[str] = text.split_whitespace().collect()
    return words

fn main():
    t = "a b c".to_upper()
    ws = words_of(t.as_str())
    t = "x".to_upper()               # the old text is dropped here
    println(ws[0])
