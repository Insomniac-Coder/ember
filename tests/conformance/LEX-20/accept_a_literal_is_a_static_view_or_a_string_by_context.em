#$ test: run-pass
#$ rules: LEX-20, TXT-9
#$ stdout: 1 2 static owned 4 view field bc
## `[LEX-20]` - a string literal is a `StrView` with the static region (so a function may return one);
## where a `String` is expected (an argument, a field, a return, an element) it produces a `String`.

struct Named:
    name: String

fn show(v: StrView) -> int:
    return v.len()

fn own(s: String) -> int:
    return s.len()

fn lit() -> StrView:
    return "static"

fn ret() -> String:
    return "owned"

fn main():
    a = "text"
    b: StrView = "view"
    n = Named("field")
    xs: Array[String] = ["a", "bc"]
    println(show("x"), own("yy"), lit(), ret(), a.len(), b, n.name, xs[1])
