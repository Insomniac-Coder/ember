# Idea 1: today a use after move is refused; proposed: share + copy on write
fn main():
    a = [1, 2, 3]
    b = a
    a.push(4)
    println(b.len())
