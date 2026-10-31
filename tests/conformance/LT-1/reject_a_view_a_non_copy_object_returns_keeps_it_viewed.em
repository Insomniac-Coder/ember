#$ test: compile-fail
#$ rules: LT-1
# `[LT-1]` — a `this` whose type is not `Copy` is the caller's place (a source of the second kind),
# and the call's view is on it: the `StrView` `word` returns keeps `w` viewed, so a second call
# while the first view lives is refused. Deriving `Copy` makes it a view of the first kind only
# (`LT-20/accept_items_of_a_copy_view_iterator_outlive_the_next_call`). A result of a view type is
# the body's summary instead (`LT-22/accept_items_of_a_non_copy_iterator_point_into_what_it_reads`).

struct Words:
    rest: StrView

    fn word() -> StrView:
        rest = this.rest
        match rest.find(" "):
            Some(at):
                this.rest = rest[at + 1..]
                return rest[..at]
            None:
                this.rest = rest[rest.len()..]
                return rest

fn main():
    w = Words(rest = "a b c")
    x = w.word()
    y = w.word() #$ error[E3025]
    println(x, y)
