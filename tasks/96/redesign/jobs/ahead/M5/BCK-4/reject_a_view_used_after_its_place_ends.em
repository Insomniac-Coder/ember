# Clause BCK-4.1: "A place whose storage ends (scope exit, `StorageDead`, its owner handed over,
# replaced or ended) while a view of it is live is `E3060` at the view's later use"
#$ test: compile-fail
#$ rules: BCK-4, OWN-3, BRW-13
# One case per way a place ends; the spec gives no message text for them (shape B7): codes only,
# each at the view's later use. Q2 (INDEX.md): a view a local of the same function holds is clause
# 1's, whatever block the local was declared in; `E3063` is for a value leaving the function's locals.

fn scope_exit():
    r: Option[ref int] = None
    if true:
        x = 5
        r = Some(ref x)
    println(r)    #$ error[E3060]

fn handed_over():
    a = [String("a")]
    v = a[0]
    b = a
    println(v)    #$ error[E3060]
    println(b)

fn replaced():
    s = String("abc")
    w: StrView = s
    s = String("xyz")
    println(w)    #$ error[E3060]
    println(s)

fn main():
    scope_exit()
    handed_over()
    replaced()
