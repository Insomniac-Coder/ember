#$ test: compile-fail
#$ rules: TYP-15, CLS-10, CLO-4
#$ profiles: debug
# `[TYP-15]` (D-547) — a class object's field and a list's element hold only
# `static` views, also when a method, `init` or a function stores through
# `self` or a reference to them: the view a caller passes must be `static`. A
# `Named` built over a local's `str` outlived the local, and its `s` read freed
# memory. A closure that borrows is a view too.

class Named:
    s: StrView = "x"

    fn init(s: StrView):
        this.s = s

    fn rename(s: StrView):
        this.s = s

[F: needs fn(int)]
class Holder:
    f: F

    fn init(owns f: F):
        this.f = f

fn put(ref mut names: Array[StrView], s: StrView):
    names[0] = s

fn main():
    names: Array[Named] = []
    if true:
        t = "abcdefgh".to_string()
        names.push(Named(t.as_str()))  #$ error[E3063]: `StrView` is a view, so it may not be passed to `init`, which stores it where only a `static` view may go
    u = "ab".to_string()
    names[0].rename(u.as_str())  #$ error[E3063]: `StrView` is a view, so it may not be passed to `rename`, which stores it where only a `static` view may go
    words: Array[StrView] = ["a"]
    put(words, u.as_str())  #$ error[E3063]: `StrView` is a view, so it may not be passed to `put`, which stores it where only a `static` view may go
    seen: Array[int] = [1, 2]
    h = Holder(fn(v: int) => println(v + seen.len()))  #$ error[E3063]: `ref Array[i64]` is a view, so it may not be passed to `init`, which stores it where only a `static` view may go
    h.f(1)
