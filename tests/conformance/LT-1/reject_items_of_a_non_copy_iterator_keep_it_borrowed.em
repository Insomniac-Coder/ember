#$ test: compile-fail
#$ rules: LT-1
# `[LT-1]` — a `mut self` whose type is not `Copy` is the caller's place (a
# source of the second kind), and the call's loan is on it: a view `next`
# returns keeps the iterator borrowed, so a second call while the first item
# lives is refused. Deriving `Copy` makes it a view of the first kind only
# (`LT-20/accept_items_of_a_copy_view_iterator_outlive_the_next_call`).

struct Words:
    rest: str

    fn next(mut self) -> Option[str]:
        rest = self.rest
        match rest.find(" "):
            Some(at):
                self.rest = rest[at + 1..]
                return Some(rest[..at])
            None:
                self.rest = rest[rest.len()..]
                return Some(rest)

fn main():
    w = Words(rest = "a b c")
    x = w.next()
    y = w.next() #$ error[E3025]
    println(x, y)
