#$ test: run-pass
#$ rules: CTL-1, CTL-2, CTL-4, TXT-10, TYP-31
#$ profiles: debug, release, shipping
#$ stdout: [0, 1, 2, 4, 7, 11]
#$ stdout: [97, 0, 233, 19990, 128512, 1114111]
#$ stdout: 6 6
#$ stdout: [0, 2, 4]
#$ stdout: empty
#$ stdout: empty owned
#$ stdout: empty view
#$ assert-c: contains("ember_str_char_next_usize(")
# Native text loops use an unsigned internal cursor, but their byte offsets
# remain int. All UTF-8 widths, NUL, and the maximum scalar advance exactly.
# The step happens before continue; break suppresses else, an empty loop runs it.

@noinline
fn signed_index(value: int) -> int:
    return value

fn main():
    text = "a\0é世😀\u{10FFFF}"
    offsets: Array[int] = []
    codes: Array[u32] = []
    for (i, c) in text.char_indices():
        offsets.push(signed_index(i))
        codes.push(c as u32)
    println(offsets)
    println(codes)

    owned_text = String.from(text)
    direct = 0
    for _ in owned_text:
        direct += 1
    chars = 0
    for _ in text.chars():
        chars += 1
    println(direct, chars)

    kept: Array[int] = []
    for (i, c) in text.char_indices():
        if c == '\0':
            continue
        if c == '😀':
            break
        kept.push(signed_index(i))
    else:
        panic("break unexpectedly ran else")
    println(kept)
    for _ in "":
        panic("an empty text yielded a character")
    else:
        println("empty")
    empty_owned = String()
    for _ in empty_owned:
        panic("an empty String yielded a character")
    else:
        println("empty owned")
    empty_bytes: Array[u8] = []
    match empty_bytes.as_span().to_str():
        Ok(empty_view):
            for _ in empty_view:
                panic("an empty byte view yielded a character")
            else:
                println("empty view")
        Err(_):
            panic("valid empty UTF-8 was rejected")
