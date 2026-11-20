# Clause THR-18.2: "`guarded by lock` after a field's type (...) names the `Lock` that must be held
# for every read and change of that field outside `init`: the field is used only inside `with
# x.lock:` on the same object `x` (...), on the thread that holds it (`E7013`"; "A function whose
# body uses a guarded field without taking its lock is worked out to need that lock held (its
# summary, `[EFF-24]`), and is called only where the lock is held (`E7013`)."; "Where the callee's
# lock may be the held one and may not be, the call is `E7018`"
#$ test: compile-fail
#$ rules: THR-18, THR-19, EFF-24
#$ error[E3060]: `log.lock = Lock()` (line 45) replaces the lock this block holds (line 44), whose end would release a lock never taken; give the log a new lock after the block, or keep the one made with it
# The spec's four messages, with this file's names and line numbers; the call of a function that
# needs the lock is code only. The `E3060` is claimed for the file: whether it stands on the
# replacing line or the block's end is not said (Q3).

from std.sync import Lock

class Stats:
    lock: Lock = Lock()
    words: int guarded by lock = 0

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(owns String(s))

fn bump_raw(s: Stats):
    s.words += 1

fn copy_into(held: Log, other: Log):
    with held.lock:
        other.push("x")    #$ error[E7018]: `other.push` takes `other`'s lock, and `other` may be `held`, whose lock this thread holds; write `if other is not held:` above line 33 and indent line 33 under it

fn main():
    stats = Stats()
    stats.words += 1    #$ error[E7013]: `words` is guarded by `lock`; change it inside `with stats.lock:`
    a = Stats()
    b = Stats()
    with a.lock:
        b.words += 1    #$ error[E7013]: `b.words` is guarded by `b.lock`, and this block holds `a.lock`; write `with b.lock:`
    bump_raw(stats)    #$ error[E7013]
    log = Log()
    with log.lock:
        log.lock = Lock()
    copy_into(log, Log())
