# A25 attack: a word kept across clear() of a list that took its lines along.
fn first_word(text: str) -> int:
    words: Array[str] = []
    for line in text.split("\n"):
        low = line.to_lower()
        for w in low.split_whitespace():
            words.push(w)
    best = words[0]
    words.clear()
    return best.len()

fn main():
    println(first_word("Alpha beta\nGamma delta"))
