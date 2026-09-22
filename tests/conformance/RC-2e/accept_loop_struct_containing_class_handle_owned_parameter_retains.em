#$ test: run-pass
#$ rules: OWN-7, RC-1, RC-2e, CTL-1, CTL-2, TYP-14
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 3
#$ stdout: 7

# `[RC-2e]` applies equally when the borrowed loop element is a value type
# containing a class handle. Reading `entry.token` for an owned call must retain
# that nested handle at the use, not for every iteration unconditionally.
class Token:
    value: i32

@derive(Copy)
struct Entry:
    token: Token

fn consume(owned token: Token) -> i32:
    return token.value

fn main():
    entries: Array[Entry] = Array[Entry]()
    entries.push(Entry(Token(7)))
    for entry in entries:
        println(consume(entry.token))
