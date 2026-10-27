#$ test: run-pass
#$ rules: R12, R4
#$ stdout: a
#$ stdout: b
#$ stdout: b
## R12 (rules.md lines 782-784): `call_once` holds the `once fn()` it uses up; `call_twice` calls a
## read-only `f` twice, allowed since the function has no state of its own to change (lines 776-779), and
## each run hands `consume` a new copy of the real `t`. AN-59's NEW program, the nearest allowed program to
## every refusal of `once fn` in this folder.

fn call_once(owns f: once fn()):
    f()

fn call_twice(f: fn()):
    f()
    f()

fn consume(owns s: String):
    println(s)

fn main():
    s = String("a")
    call_once(fn() owns s: consume(s))
    t = String("b")
    call_twice(fn(): consume(copy of t))
