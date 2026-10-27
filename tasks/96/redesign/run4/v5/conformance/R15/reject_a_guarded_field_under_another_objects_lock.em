#$ test: compile-fail
#$ rules: THR-18
## design rules: R15
## R15: the field is guarded by its own object's lock: "the lock of another object (E7013)" (AN-74).

from std.sync import Lock

class Stats:
    lock: Lock = Lock()
    words: int guarded by lock = 0

fn move_word(ref mut a: Stats, ref mut b: Stats):
    with a.lock:
        b.words += 1    #$ error[E7013]: `b.words` is guarded by `b.lock`, and this block holds `a.lock`; write `with b.lock:`

fn main():
    a = Stats()
    b = Stats()
    move_word(ref mut a, ref mut b)
