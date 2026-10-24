# A25: is the returned list of views tied to the caller's text? Use it after the text is gone.
fn words_of(text: str) -> Array[str]:
    words: Array[str] = text.split_whitespace().collect()
    return words

fn get_words() -> Array[str]:
    t = "a b c".to_upper()
    return words_of(t.as_str())      # the words view t, which ends here

fn main():
    println(get_words().len())
