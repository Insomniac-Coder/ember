# Clause WK-7.1: "Refused: an opening that depends on what is only known while running, and a loop
# handed out of reach (returned, stored where the compiler cannot follow it)."
#$ test: compile-fail
#$ rules: WK-7, WK-6
# An opening under a condition known only while running, a loop returned, and a loop stored in a
# module variable. No text in the spec for these: code only, at the closing line.

class Node:
    name: String
    next: Option[shares mut Node] = None

static mut KEPT: Array[shares mut Node] = []

fn maybe_opened(flag: bool):
    a = shares mut Node(name=String("a"))
    b = shares mut Node(name=String("b"))
    a.next = Some(shares mut b)
    b.next = Some(shares mut a)    #$ error[E3032]
    if flag:
        b.next = None

fn returned() -> shares mut Node:
    a = shares mut Node(name=String("a"))
    b = shares mut Node(name=String("b"))
    a.next = Some(shares mut b)
    b.next = Some(shares mut a)    #$ error[E3032]
    return a

fn stored():
    a = shares mut Node(name=String("a"))
    b = shares mut Node(name=String("b"))
    a.next = Some(shares mut b)
    b.next = Some(shares mut a)    #$ error[E3032]
    KEPT.push(shares mut a)

fn main():
    maybe_opened(input() == "y")
    _r = returned()
    stored()
