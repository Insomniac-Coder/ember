#$ test: run-pass
#$ rules: CLO-12, CLO-2, GRM-28
#$ stdout: 4
#$ stdout: 6
#$ stdout: 7
## `[CLO-12]` — a local function is a named closure: one that writes a capture
## borrows it mutably, and calling itself passes that borrow on; one in a
## method captures `self`.

struct Counter:
    n: int

    fn total(self, extra: int) -> int:
        fn plus(k: int) -> int:
            return self.n + k
        return plus(extra)

fn main():
    count = 0
    fn walk(n: int):
        count += 1
        if n > 0:
            walk(n - 1)
    walk(3)
    println(count)
    items = [3, 1, 2]
    fn sum_from(i: int) -> int:
        if i >= items.len():
            return 0
        return items[i] + sum_from(i + 1)
    println(sum_from(0))
    println(Counter(4).total(3))
