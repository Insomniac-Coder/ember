# Clause WK-7.3: "**A store is judged in the package that can see both ends.** A store whose stored
# value is a parameter (...) is not judged in the body: the function's summary records which
# parameter (...) it stores into a place of which other parameter or of `this`, (...) and each call
# of the function is that store"; "A function value given as the argument is the stored value, and
# its state's owners are its edges (`[WK-5]`): `p.btn.on_click(owns fn() owns q: println(q.name))`
# (...) closes `p` → `btn` → the handler's `q` → `p` and is `E3032` at the call."
#$ test: compile-fail
#$ rules: WK-7, WK-5, CLO-17, EFF-24
# `link` compiles; its second call closes the loop. The spec's panel and button. No text in the spec
# for these programs: code only, at the calls.

class Node:
    name: String
    next: Option[shares mut Node] = None

fn link(ref mut x: shares mut Node, owns y: shares mut Node):
    x.next = Some(owns y)

class Button:
    handler: fn() = noop

    fn on_click(owns f: fn()):
        this.handler = f

class Panel:
    name: String
    btn: Button = Button()

fn noop():
    pass

fn main():
    a = shares mut Node(name=String("a"))
    b = shares mut Node(name=String("b"))
    link(ref mut a, shares mut b)
    link(ref mut b, shares mut a)    #$ error[E3032]
    p = shares mut Panel(name=String("p"))
    q = shares mut p
    p.btn.on_click(owns fn() owns q: println(q.name))    #$ error[E3032]
