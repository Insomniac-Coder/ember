#$ test: compile-fail
#$ rules: TYP-15, CLS-10, CLO-4
#$ profiles: debug
# `[TYP-15]` (D-547) — a class object's field and a list's element hold only
# `static` views, also when a method, `init` or a function stores through
# `self` or a reference to them: the view a caller passes must be `static`. A
# `Named` built over a local's `str` outlived the local, and its `s` read freed
# memory. A closure that borrows is a view too.

class Named:
    s: str = "x"

    fn init(mut self, s: str):
        self.s = s

    fn rename(mut self, s: str):
        self.s = s

class Holder[F: fn(int)]:
    f: F

    fn init(mut self, owned f: F):
        self.f = f

fn put(mut names: Array[str], s: str):
    names[0] = s

fn main():
    names: Array[Named] = []
    if true:
        t = "abcdefgh".to_string()
        names.push(Named(t.as_str()))  #$ error[E3063]: `str` is a view, so it may not be passed to `init`, which stores it where only a `static` view may go
    u = "ab".to_string()
    names[0].rename(u.as_str())  #$ error[E3063]: `str` is a view, so it may not be passed to `rename`, which stores it where only a `static` view may go
    words: Array[str] = ["a"]
    put(words, u.as_str())  #$ error[E3063]: `str` is a view, so it may not be passed to `put`, which stores it where only a `static` view may go
    seen: Array[int] = [1, 2]
    h = Holder(fn(v: int) => println(v + seen.len()))  #$ error[E3063]: `ref Array[i64]` is a view, so it may not be passed to `init`, which stores it where only a `static` view may go
    h.f(1)
