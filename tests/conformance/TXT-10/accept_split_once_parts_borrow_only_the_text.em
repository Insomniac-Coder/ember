#$ test: run-pass
#$ rules: TXT-10, LT-22, BRW-1
#$ profiles: debug, release, shipping
#$ stdout: left right
#$ stdout: 0 0
# Both sides borrow the searched text. A scoped separator may be destroyed,
# and a named separator mutated, while either returned side remains live.

fn separator() -> String:
    return ":"

fn main():
    text = String.from("left:right")
    needle = String.from(":")
    pair = text.split_once(needle.as_str())
    needle.clear()
    match pair:
        Some((before, after)):
            println(before, after)
        None:
            panic("separator unexpectedly missing")
    if true:
        temporary = separator()
        pair = text.split_once(temporary.as_str())
    match pair:
        Some((before, after)):
            assert(before == "left" and after == "right")
        None:
            panic("temporary separator unexpectedly missing")
    text.clear()
    println(text.len(), needle.len())
