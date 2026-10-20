#$ test: run-pass
#$ rules: STD-7
#$ stdout: true true true
#$ stdout: x y 2 true 1
#$ stdout: true true true héllo! 7
#$ stdout: Some(0) true false 3 Some(1) Some(9)
#$ stdout: 4
## `[STD-7]` — `FixedArray[T, N]`, `FixedString[N]` and `RingBuffer[T, N]` hold their values inline:
## `push` past the capacity is `Err(CapacityError.Full)` and changes nothing, and `push_or_drop`
## drops the value instead. Pushing allocates nothing, so `@noalloc` code may do it.

from std.collections import FixedArray, FixedString, RingBuffer

@noalloc
fn fill(mut a: FixedArray[int, 4]) -> int:
    for i in 0..10:
        a.push_or_drop(i)
    return a.len()

fn main():
    a = FixedArray[String, 2].new()
    println(a.push(String.from("x")).is_ok(), a.push(String.from("y")).is_ok(), a.push(String.from("z")).is_err())
    a.push_or_drop(String.from("w"))
    line = String()
    for s in a.iter():
        line.push_str(s)
        line.push_str(" ")
    print(line)
    println(a.len(), a.pop() == Some(String.from("y")), a.len())

    t = FixedString[8].new()
    println(t.push_str("héllo").is_ok(), t.push_str("1234").is_err(), t.push('!').is_ok(), t.as_str(), t.len())

    r = RingBuffer[int, 3].new()
    for i in 0..5:
        r.push_or_drop(i)
    println(r.pop(), r.push(9).is_ok(), r.push(10).is_ok(), r.len(), r.get(0), r.get(2))

    numbers = FixedArray[int, 4].new()
    println(fill(numbers))
