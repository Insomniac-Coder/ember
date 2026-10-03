#$ test: run-pass
#$ rules: TXT-10, LT-20, LT-22, LT-36, LT-38, BRW-1
#$ profiles: debug, release, shipping
#$ stdout: left right
#$ stdout: 0
#$ stdout: some 0
#$ stdout: Some('é') Some('tail') Some('') None None
#$ stdout: Some('é') Some('tail') None
# The wrapper returns the new builtin's nested tuple provenance. Its text
# is a named borrowed view, and its separator is cleared and dropped before
# either returned field is used. Reading only a result tag after discarding
# both payload fields permits source mutation. Stored Split.next results
# coexist across later calls, a trailing empty item, and repeated exhaustion.

@noinline
fn split_view(text: str, separator: str) -> Option[(str, str)]:
    return text.split_once(separator)

fn main():
    text = String.from("left:right")
    view = text.as_str()
    pair: Option[(str, str)] = None
    if true:
        separator = String.from(":")
        pair = split_view(view, separator.as_str())
        separator.clear()
    match pair:
        Some((before, after)):
            println(before, after)
        None:
            panic("wrapper separator unexpectedly missing")
    text.clear()
    println(text.len())

    tagged_text = String.from("red:blue")
    tagged = tagged_text.split_once(":")
    match tagged:
        Some((_, _)):
            pass
        None:
            panic("tag separator unexpectedly missing")
    tagged_text.clear()
    match tagged:
        Some(_):
            println("some", tagged_text.len())
        None:
            println("none", tagged_text.len())

    items_text = String.from("é🦀tail🦀")
    items = items_text.split("🦀")
    first = items.next()
    second = items.next()
    trailing = items.next()
    exhausted = items.next()
    exhausted_again = items.next()
    println(first, second, trailing, exhausted, exhausted_again)
    println(first, second, items.next())
