#$ test: run-pass
#$ rules: LT-1, LT-22
#$ stdout: a b
# `[LT-1]`, `[LT-22]` (verdict 137) — the code of the function called says what its result points
# into, for a plain `StrView` as for a result of a view type
# (`LT-22/accept_items_of_a_non_copy_iterator_point_into_what_it_reads`): `word` returns a piece of
# `this.rest`, the text `w` views, not `w` itself, so the second call while `x` lives is no conflict.

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
    y = w.word()
    println(x, y)
