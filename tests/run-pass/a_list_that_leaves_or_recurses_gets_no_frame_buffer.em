#$ test: run-pass
#$ rules: ALC-1
#$ stdout: 1225
#$ stdout: item-7
#$ stdout: 3
#$ assert-c: !contains("stack_vec(_sb")

## ADR-141 — no frame buffer for a list that leaves its function (returned,
## or moved into an object's field), nor in a function that may run again
## while it runs (a recursion's every frame would carry one).

fn count_down(n: int) -> int:
    label = f"level {n}"
    if n == 0:
        return label.len() - 7
    return n + count_down(n - 1)

fn make_label(n: int) -> String:
    s = f"item-{n}"
    return s

class Holder:
    text: String

    fn init(mut self):
        self.text = String()

fn main():
    println(count_down(49))
    println(make_label(7))
    h = Holder()
    t = String()
    for _ in 0..3:
        t.push('a')
    h.text = t
    println(h.text.len())
