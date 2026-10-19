#$ test: run-pass
#$ rules: EXP-4, DRP-3
#$ profiles: debug, development
#$ stdout: 10
#$ still live
# `[EXP-4]` (D-673) — a statement's temporaries drop "in reverse order of creation", and a temporary
# is created when its value is complete: `hold(text())`'s `Holder`, made from the `String` that
# `text()` returns, drops first, while the `String` it views is still there. Ordered by when their
# slots were taken, the `String` dropped first and the program was `E3060`.

struct Holder:
    borrowed: str
    marker: i32

extend Holder:
    fn drop(mut self):
        println(self.borrowed)

fn text() -> String:
    return String.from("still live")

fn hold(s: str) -> Holder:
    return Holder(s, 0)

fn width(h: Holder) -> int:
    return h.borrowed.char_count()

fn main():
    println(width(hold(text())))
