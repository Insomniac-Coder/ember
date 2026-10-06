fn make_text() -> String:
    text = String.from("")
    for i in 0..50000:
        text.push_str("the quick brown fox, jumps over\tthe lazy dog; héllo wörld\n")
    return text

fn main():
    text = make_text()
    total = 0
    for round in 0..20:
        for c in text.chars():
            total += c as u32 as int
    println(total)
