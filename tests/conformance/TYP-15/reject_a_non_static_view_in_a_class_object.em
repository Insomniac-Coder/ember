#$ test: compile-fail
#$ rules: TYP-15, LT-3
# `[TYP-15]` (D-353) — a class object's fields are not bounded by any local's
# region, so a view stored in one must be `static`: a string literal is, a
# view of a local `String` is not — whether it is stored by the constructor,
# by an assignment, or by a function given it.

class Label:
    text: str

fn make(t: str) -> Label:
    return Label(t)

fn main():
    ok = Label("fixed")
    ok.text = "also fixed"
    s = String.from("zed")
    bad = Label(s.as_str()) #$ error[E3063]: `str` is a view, so it may not be stored in a class object unless it is `static`
    ok.text = s.as_str() #$ error[E3063]: `str` is a view, so it may not be stored in a class object unless it is `static`
    made = make(s.as_str()) #$ error[E3063]: `str` is a view, so it may not be passed to `make`, which stores it where only a `static` view may go
    println(ok.text, bad.text, made.text, make("lit").text)
