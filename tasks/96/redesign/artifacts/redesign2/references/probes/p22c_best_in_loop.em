# A22: the view of a per-iteration temporary kept after the loop (Python: best = line.lower().strip()).
fn longest(lines: Array[String]) -> int:
    best: str = ""
    for line in lines:
        w = line.to_lower().trim()
        if w.len() > best.len():
            best = w
    return best.len()

fn main():
    println(longest(["a ", " bb"]))
