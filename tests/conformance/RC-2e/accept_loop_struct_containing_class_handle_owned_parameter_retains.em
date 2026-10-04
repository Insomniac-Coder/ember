#$ test: run-pass
#$ rules: OWN-7, RC-1, RC-2e, CTL-1, CTL-2, TYP-14, RNG-4
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 2
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 7

# `[RC-2e]` applies equally when the borrowed loop element is a value type
# containing a class handle. Reading `entry.token` for an owned call must retain
# that nested handle at the use, not for every iteration unconditionally.
# Range facts prove the loop's index in bounds (`[RNG-4]`), so the loop is
# not versioned: its body, and the retains in it, appear once.
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
