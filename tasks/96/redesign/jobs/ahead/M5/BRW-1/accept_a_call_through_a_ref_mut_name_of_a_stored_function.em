# Clause BRW-1.7: "A call through a `ref mut` name of a stored function value uses the name only to
# copy the function's two words out, before the arguments are given (`[CLO-10]`), so the call's
# arguments may reach the place it names; a later use of the name is checked against what the call
# may have done."
#$ test: run-pass
#$ rules: BRW-1, CLO-10, BRW-10, FN-2a
#$ stdout: 2
# `h` names `b.on_click`, and `ref mut b` reaches it: the call copies `count` out of `h` first.
# `count` changes only `b.clicks`, so `h` may be used again after the first call.

struct Button:
    clicks: int
    on_click: fn(ref mut Button)

fn count(ref mut b: Button):
    b.clicks += 1

fn main():
    b = Button(clicks=0, on_click=count)
    h = ref mut b.on_click
    h(ref mut b)
    h(ref mut b)
    println(b.clicks)
