fn make_text() -> String:
    text = String.from("")
    for i in 0..50000:
        text.push_str("the quick brown fox, jumps over\tthe lazy dog; héllo wörld\n")
    return text

fn main():
    text = make_text()
    count = 0
    length = 0
    for round in 0..200:
        for line in text.lines():
            count += 1
            length += line.len()
    println(count, length)
