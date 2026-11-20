# Clause EFF-27.1: "A change that does not pass through the method's object cannot be cleared that
# way, so a `sharedref mut` link, or a `shares mut` owner, made into anything a `@cached` method of
# its holder reads is `E2117` at the line that makes it"
#$ test: compile-fail
#$ rules: EFF-27, CLS-12, LT-46
# The spec's message, with this file's names; the link is kept in a `Watcher` (`[BRW-17]`'s form).

struct Book:
    name: String

class Shelf:
    items: Array[Book] = []

    @cached
    fn first_name() -> String:
        return copy of this.items[0].name

class Watcher:
    target: sharedref mut Book

fn main():
    s = Shelf()
    s.items.push(owns Book(String("a")))
    println(s.first_name())
    w = Watcher(target=sharedref mut s.items[0])    #$ error[E2117]: `s.items[0]` is read by `Shelf.first_name`, which is `@cached` and would not see a change made through this link; link the shelf instead, `sharedref mut s`, or remove `@cached`
    w.target.name = String("b")
