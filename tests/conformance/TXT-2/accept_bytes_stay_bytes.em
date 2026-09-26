#$ test: run-pass
#$ rules: TXT-2, TXT-1, STD-15, SPN-2, TYP-39
#$ stdout: [104, 105, 33] 3 104
#$ stdout: [104, 105] 2
#$ stdout: [[104, 105, 33]] 242
#$ stdout: [33, 104, 105] true
#$ stdout: 33 104 105
#$ stdout: hi! hi ['hi!'] 3
# `[TXT-1]` — a `String` is UTF-8 text and an `Array[u8]` is an `Array` like any other: it
# prints, indexes, slices (to a `Span[u8]`), sorts and iterates as bytes, and generic code over
# `Array[T]` takes it. The two were one type in the compiler, so bytes sliced as text and printed
# quoted (D-201).

fn total[T: Add[Output = T] + Default + Copy](xs: Array[T]) -> T:
    s = T.default()
    for x in xs:
        s = s + x
    return s

fn main():
    bytes: Array[u8] = [104, 105, 33]
    println(bytes, len(bytes), bytes[0])
    head = bytes[0..2]
    println(head, head.len())
    println([bytes.clone()], total(bytes))
    bytes.sort()
    println(bytes, bytes.contains(33))
    line: String = ""
    for b in bytes:
        line = f"{line} {b}" if line != "" else f"{b}"
    println(line)
    s: String = "hi!"
    println(s, s[0..2], [s.clone()], len(s.as_bytes()))
