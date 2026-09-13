fn main():
    arena = Arena.with_capacity(64)
    values: Array[i32] = Array[i32]()
    stored = arena.alloc(values)
    println(stored.len())

