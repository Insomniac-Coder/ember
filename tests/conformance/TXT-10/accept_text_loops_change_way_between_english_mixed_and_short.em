#$ test: run-pass
#$ rules: TXT-10, TXT-2
#$ profiles: debug, development, release
#$ stdout: 945763144 254 0 -1 255
#$ stdout: 318657419 255 0 -1 256
#$ stdout: 458003899 400 -1 -1 400
#$ stdout: 99054247 400 495 300 500
#$ stdout: 596435079 120 357 300 360
#$ stdout: 941168544 432 463 316 466
#$ stdout: 615597433 302 600 300 605
#$ stdout: 304409929 301 300 300 303
#$ stdout: 430460717 301 0 -1 303
#$ stdout: 789465869 300 597 301 600

## D-550 (ADR-199) — gcc's text loop has three ways: the English loop (ASCII steps), the mixed loop
## (taken at a wide character that starts within 4 bytes of the last one's end, left after 32 ASCII
## characters in a row) and the short loop (a text under 256 bytes). These texts cross between them:
## 255 and 256 bytes, long ASCII, wide characters 3 bytes apart and back to back, 40 ASCII
## characters inside mixed text, four-byte characters, a wide character first and last. Each loop
## body (a total, a count, a condition, an early return) gives what the decoder alone gives; the
## expected values were computed by Python's UTF-8 decoding.

@noinline
fn sum(text: str) -> int:
    total = 0
    for (i, c) in text.char_indices():
        total = total * 31 + (i ^ (c as u32 as int))
        total = total % 1000000007
    return total

@noinline
fn count(text: str) -> int:
    n = 0
    for _c in text:
        n += 1
    return n

@noinline
fn last_wide(text: str) -> int:
    at = -1
    for (i, c) in text.char_indices():
        if c as u32 >= 128:
            at = i
    return at

@noinline
fn first_after(text: str, start: int) -> int:
    for (i, c) in text.char_indices():
        if i >= start and c as u32 >= 128:
            return i
    return -1

fn build(head: str, body: str, n: int, tail: str) -> String:
    out = String.from(head)
    for _k in 0..n:
        out.push_str(body)
    out.push_str(tail)
    return out

fn main():
    texts: Array[String] = [
        build("é", "a", 253, ""),
        build("é", "a", 254, ""),
        build("", "the quick brown fox ", 20, ""),
        build("", "é a ", 100, ""),
        build("", "敏捷棕色", 30, ""),
        build("", "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxéàü", 10, "中文"),
        build("🙂", "ab🙂", 100, "z"),
        build("", "abc", 100, "€"),
        build("€", "abc", 100, ""),
        build("", "a€", 150, "")
    ]
    for t in texts:
        println(sum(t), count(t), last_wide(t), first_after(t, 300), t.len())
