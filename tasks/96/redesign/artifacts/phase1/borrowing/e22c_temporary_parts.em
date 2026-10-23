# A22: split a temporary and keep the parts.
fn parts(line: str) -> int:
    pieces: Array[str] = line.to_lower().split(" ").collect()
    return pieces.len()

fn main():
    println(parts("A b"))
