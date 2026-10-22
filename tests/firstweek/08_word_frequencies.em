# Count word frequencies in a paragraph, ignoring case and punctuation, and print the ten most common words.

const TEXT: str = """
The river was low that summer, and the children walked along the river bed
every morning. They found stones, and shells, and once a rusty key. The key
opened nothing, but the children kept it anyway, because a key that opens
nothing might open anything. In the evening the river came back a little, and
the stones they had not taken were under the water again. Nobody minded.
The summer was long, the key was small, and the river did not care.
"""

fn clean_word(word: str) -> String:
    result = ""
    for c in word.lower():
        if c.isalpha() or c == "'":
            result += c
    return result

fn count_words(text: str) -> Map[String, int]:
    counts: Map[String, int] = {}
    for raw in text.split():
        word = clean_word(raw)
        if word == "":
            continue
        if word in counts:
            counts[word] += 1
        else:
            counts[word] = 1
    return counts

fn main():
    counts = count_words(TEXT)
    total = sum(counts.values())
    println(f"{total} words, {counts.len()} distinct")

    pairs = [(word, n) for word, n in counts.items()]
    pairs.sort(key=fn(p) => (-p[1], p[0]))
    println("top ten:")
    for i, (word, n) in pairs[:10].iter().enumerate():
        bar = "#" * n
        println(f"{i + 1:>2}. {word:<10} {n:>3} {bar}")

    longest = ""
    for word in counts.keys():
        if word.len() > longest.len():
            longest = word
    println(f"longest word: {longest}")

    once = [w for w, n in counts.items() if n == 1]
    println(f"{once.len()} words appear only once")

    by_length: Map[int, int] = {}
    for word, n in counts.items():
        by_length[word.len()] = by_length.get(word.len(), 0) + n
    for length in sorted(by_length.keys()):
        println(f"{length:>2} letters: {by_length[length]}")
