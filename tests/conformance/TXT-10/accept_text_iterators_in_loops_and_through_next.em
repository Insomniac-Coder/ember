#$ test: run-pass
#$ rules: TXT-10, CTL-1, CTL-4
#$ profiles: debug, release, shipping
#$ stdout: ['a', 'é', '😀', 'b'] ['a', 'é', '😀', 'b'] 4 Some('b')
#$ [0, 1, 3, 7] [(0, 'a'), (1, 'é'), (3, '😀'), (7, 'b')]
#$ 1238 8 [97, 195, 169, 240, 159, 152, 128, 98]
#$ [one][two][][three] 1 0 Some('two')
#$ [a][][b][][c][] ['1', '2', '3'] 1
#$ [a][b][c] 0
#$ a😀
#$ empty
# `[TXT-10]` — `chars`, `char_indices`, `bytes`, `lines`, `split` and
# `split_whitespace`. In a `for` header the first three are counted loops
# (ADR-106) and the rest call `next`; an iterator held in a variable, or an
# adapter's, is driven by `next`. Both give the same items, and `[CTL-4]`'s
# `else`, `break` and `continue` work in either.

fn joined(parts: Array[str]) -> String:
    out = String.from("")
    for part in parts:
        out += f"[{part}]"
    return out

fn main():
    s = "aé😀b"
    # In a `for` header each is a counted loop; held in a variable, the
    # iterator is driven by `next`. Both give the same items.
    looped: Array[char] = []
    for c in s.chars():
        looped.push(c)
    driven: Array[char] = []
    it = s.chars()
    for c in it:
        driven.push(c)
    println(looped, driven, s.chars().count(), s.chars().last())
    offsets: Array[int] = []
    for (i, _) in s.char_indices():
        offsets.push(i)
    println(offsets, s.char_indices().to_array())
    total = 0
    for b in s.bytes():
        total += b as int
    println(total, s.bytes().count(), s.bytes().to_array())
    text = "one\ntwo\r\n\nthree"
    lines: Array[str] = []
    for line in text.lines():
        lines.push(line)
    println(joined(lines), "x\n".lines().count(), "".lines().count(), text.lines().nth(1))
    parts: Array[str] = []
    for part in "a,,b,,c,".split(","):
        parts.push(part)
    println(joined(parts), "1é2é3".split("é").to_array(), "".split(",").count())
    words: Array[str] = []
    for word in "  a \t b\u{3000}c  ".split_whitespace():
        words.push(word)
    println(joined(words), "   ".split_whitespace().count())
    # A loop of each kind ends early, skips and runs its `else`.
    for c in s.chars():
        if c == 'é':
            continue
        if c == 'b':
            break
        print(c)
    else:
        print("!")
    println()
    for b in "".bytes():
        print(b)
    else:
        println("empty")
