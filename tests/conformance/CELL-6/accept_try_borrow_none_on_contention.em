#$ test: run-pass
#$ rules: CELL-6, CELL-6a
# `[CELL-6]` — `try_borrow` returns `Option[Ref[T]]` instead of panicking:
# `None` on contention (here, a live `borrow_mut`), `Some` run through `match`.
# `[CELL-6a]` — `None` in every profile: the lowering has no profile input, so
# no profile can make this infallible (`[PRF-1]` forbids changing the arm).

fn main():
    c: RefCell[i32] = RefCell(1)
    with _a = c.borrow_mut():
        m = c.try_borrow()
        match m:
            Some(_g):
                println(1)
            None:
                println(2)
#$ stdout: 2
