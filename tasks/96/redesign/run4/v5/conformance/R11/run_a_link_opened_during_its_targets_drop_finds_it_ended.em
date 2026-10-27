#$ test: run-pass
#$ rules: R11, R13, R5
#$ stdout: made
#$ stdout: ended
## R11 (verdict 56; safety validation (b), hole 4): "An object is ended the moment its last owner lets
## it go, before its `drop` runs ... a `weakref` link opened during its `drop` ... finds it ended, so the
## `if ... exists as p:` block is skipped and the half-ended object is never read, nor counted again".
## The player links to itself; when `main`'s `p` ends, its `drop` opens that link and finds it ended,
## as Rust's `Weak::upgrade` does. Section 2: the header test finds it gone, with nothing more run.
## Checked under AddressSanitizer: no report (no count goes 0 to 1 and back, no second end).

class Player:
    hp: int = 100
    me: weakref Player = None

    fn drop():
        if this.me exists as q:
            println("open", q.hp)
        else:
            println("ended")

fn main():
    p = Player()
    p.me = weakref p
    println("made")
