#$ test: run-pass
#$ rules: LT-22, LT-1
#$ stdout: Some('a') Some('b')
# `[LT-22]` — a result of a view type (`Option[StrView]`) points where the body's summary says: into
# what `this.rest` points into, the text, not into `w` itself, so the first item lives on across the
# second call. A plain `StrView` result keeps `w` viewed instead (`[LT-1]`,
# `LT-1/reject_a_view_a_non_copy_object_returns_keeps_it_viewed`).

struct Words:
    rest: StrView

    fn next() -> Option[StrView]:
        rest = this.rest
        match rest.find(" "):
            Some(at):
                this.rest = rest[at + 1..]
                return Some(rest[..at])
            None:
                this.rest = rest[rest.len()..]
                return Some(rest)

fn main():
    w = Words(rest = "a b c")
    x = w.next()
    y = w.next()
    println(x, y)
