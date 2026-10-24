# A25 attack: the word list built line by line. Each lowered line dies at the end of its iteration.
fn count_all(text: str) -> int:
    all_words: Array[str] = []
    for line in text.split("\n"):
        low = line.to_lower()
        for w in low.split_whitespace():
            all_words.push(w)
    return all_words.len()

fn main():
    println(count_all("The cat\nSAT on\nthe Mat"))
