fn main():
    callbacks: Array[fn() -> int] = []
    for i in 0..3:
        name = f"item {i}"
        callbacks.push(owns owned fn() => name.len())
    println(callbacks.len())
