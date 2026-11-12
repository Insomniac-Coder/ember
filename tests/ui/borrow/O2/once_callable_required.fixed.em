fn consume(owns values: Array[i32]) -> i32:
    return values[0]

fn invoke(f: once fn() -> i32) -> i32:
    return f()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 7)
    task = owned fn() => consume(owns values)
    println(invoke(owns task))
