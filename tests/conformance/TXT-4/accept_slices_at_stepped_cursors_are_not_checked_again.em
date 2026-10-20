#$ test: run-pass
#$ rules: TXT-4
#$ profiles: debug, development, release
#$ assert-c-count: contains("ember_str_is_char_boundary(") == 0
#$ assert-c-count: contains("ember_panic_bounds(") == 0
#$ stdout:
#$ 5 16
#$ héllo|wörld|a|bc|d

## `[TXT-4]` (D-759) — a slice's bounds are checked against the end and for a character boundary,
## but a bound that is one by construction is not checked again: `0`, the text's length, and a
## cursor `next_char` stepped from one of those (whole characters, never past the end), and a word's
## end at or after its start. `split_whitespace` slices only at such cursors, so its C keeps no
## check.

fn main():
    text = "héllo  wörld a bc d "
    out = String.from("")
    count = 0
    total = 0
    for word in text.split_whitespace():
        if count > 0:
            out.push_str("|")
        out.push_str(word)
        count += 1
        total += word.len()
    println(count, total)
    println(out)
