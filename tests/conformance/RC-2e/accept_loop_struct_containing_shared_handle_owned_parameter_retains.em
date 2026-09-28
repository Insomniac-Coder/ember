#$ test: run-pass
#$ rules: OWN-7, HEAP-3, HEAP-4, HEAP-6, RC-1, RC-2e, CTL-1, CTL-2, TYP-14, RNG-4
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 3
#$ stdout: 7

# `[RC-2e]` also covers a borrowed value-type loop element with a Shared field.
# Projecting `entry.token` is borrowed from the Array element until the owned
# call boundary retains precisely that field for the callee.
# Range facts prove the loop's index in bounds (`[RNG-4]`), so the loop is
# not versioned: its body, and the retains in it, appear once.
struct Token:
    value: i32

@derive(Copy)
struct Entry:
    token: Shared[Token]

fn consume(owned token: Shared[Token]) -> i32:
    value = token.get()
    return value.value

fn main():
    entries: Array[Entry] = Array[Entry]()
    entries.push(Entry(Shared(Token(7))))
    for entry in entries:
        println(consume(entry.token))
