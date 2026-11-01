#$ test: run-pass
#$ rules: STD-19
#$ stdout: 1
#$ 2
#$ 5
# ODR-240 — the standard library's functions that take a function borrow a named one, so it may be
# called again after (OPEN ODR N36: `[STD-19]`'s 0.9.15 text says the function is handed over).

fn main():
    xs = [1, 2]
    h = fn(x: int): println(x)
    xs.iter().for_each(owns h)
    h(5)
