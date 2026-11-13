# Clause SPN-1.3: "**The conversion makes a view** of the source place with the ordinary rules: the
# source is not changed through another name, handed over or ended while the view is used
# (`[BRW-1]`), whether the conversion is written implicitly, as `as_span()`/`as_mut_span()`, or
# produced by a call."
#$ test: compile-fail
#$ rules: SPN-1, BRW-1, BRW-17, BCK-3, BCK-4, LT-1
# One function per way and per spelling; the spec gives no message text for them: codes only. An
# element written while a read-only view reads it is `E3021`, at the change; a push or a text added
# to reshapes the source and ends the view, and handing it over or ending it ends the view too:
# `E3060`, at the view's later use (`[BCK-3]`, `[BCK-4]`). While a `ref mut` view is used, the source
# is read (`E3021`) or changed (`E3022`) through no other name.

fn head(xs: Span[int]) -> Span[int]:
    return xs[0..1]

fn written_implicitly():
    xs = [1, 2, 3]
    v: Span[int] = xs
    xs[0] = 9    #$ error[E3021]
    println(v[0])

fn written_as_as_span():
    xs = [1, 2, 3]
    v = xs.as_span()
    xs[0] = 9    #$ error[E3021]
    println(v[0])

fn produced_by_a_call():
    xs = [1, 2, 3]
    v = head(xs)
    xs[0] = 9    #$ error[E3021]
    println(v[0])

fn reshaped():
    xs = [1, 2, 3]
    v = xs.as_span()
    xs.push(owns 4)
    println(v[0])    #$ error[E3060]

fn text_added_to():
    s = String("ab")
    w: StrView = s
    s.push_str("c")
    println(w)    #$ error[E3060]

fn handed_over():
    xs = [1, 2, 3]
    v: Span[int] = xs
    ys = xs
    println(v[0])    #$ error[E3060]
    println(ys)

fn ended():
    v: Option[Span[int]] = None
    if true:
        xs = [1, 2, 3]
        v = Some(xs.as_span())
    if v exists as s:    #$ error[E3060]
        println(s[0])

fn read_beside_as_mut_span():
    xs = [1, 2, 3]
    d = ref mut xs.as_mut_span()
    println(xs[1])    #$ error[E3021]
    d[0] = 0

fn changed_beside_as_mut_span():
    xs = [1, 2, 3]
    d = ref mut xs.as_mut_span()
    xs[1] = 5    #$ error[E3022]
    d[0] = 0

fn main():
    written_implicitly()
    written_as_as_span()
    produced_by_a_call()
    reshaped()
    text_added_to()
    handed_over()
    ended()
    read_beside_as_mut_span()
    changed_beside_as_mut_span()
