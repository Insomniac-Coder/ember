#$ test: run-pass
#$ rules: CTL-1, TXT-10, TYP-31
#$ profiles: debug, release, shipping
#$ stdout: [0, 1, 2, 4, 7, 11] [0, 1, 2, 4, 7, 11]
#$ stdout: [97, 0, 233, 19990, 128512, 1114111]
#$ stdout: 2 0
#$ assert-c: contains("ember_str_char_next_usize(")
#$ assert-c: contains("ember_str_char_next(")
# Stored iterators and explicit next keep std's int cursor and int offsets.
# Native iteration uses its separately typed unsigned helper, with no pointer
# cast between the two representations.

@noinline
fn signed_index(value: int) -> int:
    return value

fn main():
    text = "a\0é世😀\u{10FFFF}"
    native: Array[int] = []
    for (i, _) in text.char_indices():
        native.push(signed_index(i))
    driven: Array[int] = []
    codes: Array[u32] = []
    iterator = text.char_indices()
    for _ in 0..6:
        match iterator.next():
            Some((i, c)):
                driven.push(signed_index(i))
                codes.push(c as u32)
            None:
                panic("iterator ended before its sixth scalar")
    match iterator.next():
        Some(_):
            panic("iterator yielded past its last scalar")
        None:
            pass
    println(native, driven)
    println(codes)

    characters = "é😀".chars()
    count = 0
    for _ in characters:
        count += 1
    empty = "".chars()
    empty_count = 0
    for _ in empty:
        empty_count += 1
    println(count, empty_count)
