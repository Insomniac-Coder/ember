fn size(xs: ref Array[i32]) -> usize:
    return xs.len()

fn fill(ready: bool) -> usize:
    values: Array[i32]
    if ready:
        values = Array[i32]()
    return size(ref values)

fn main():
    println(fill(true))
