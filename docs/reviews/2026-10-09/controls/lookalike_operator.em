fn add[T: Add[Output = T]](a: T, b: T) -> T:
    return a + b

fn main():
    println(add(2, 3))
