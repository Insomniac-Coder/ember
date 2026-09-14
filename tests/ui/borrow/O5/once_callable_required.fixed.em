fn consume(owned values: Array[i32]) -> i32:
    return values[0]

fn invoke(owned f: fn() -> i32) -> i32:
    return f()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    task = owned fn() => consume(values)
    println(invoke(task))
