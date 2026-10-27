#$ test: run-pass
#$ rules: R12, R3, R4, R9
#$ stdout: 1
## R12 (rules.md lines 756-757, 776-777), verdicts 17 and 32: the E3026 message of the nearest refused
## program followed: the handler owns a share of the counter (`with owns count`, its own state), so it
## outlives nothing it uses; it changes a `Counter` through that owner, named by type. `register` only
## stores the handler (no clause); `main` clicks and names `Counter`.

class Counter:
    n: int = 0

class Button:
    handlers: Array[fn() changes Counter] = []

    fn on_click(owns f: fn() changes Counter) changes handlers:
        this.handlers.push(f)

    fn click() changes handlers, Counter:
        for ref mut h in this.handlers:
            h()

fn register(ref mut b: Button, owns count: shares mut Counter):
    b.on_click(fn() with owns count changes Counter: count.n += 1)

fn main() changes Counter:
    count = Counter()
    b = Button()
    register(ref mut b, shares mut count)
    b.click()
    println(count.n)
