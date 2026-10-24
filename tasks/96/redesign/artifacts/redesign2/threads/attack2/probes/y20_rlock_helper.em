# Proposed syntax, readability attack. Python's re-entrant lock habit: a method takes the lock and calls
# another public method that also takes it (Python writes threading.RLock() for this). The answers report
# (4.5): "Calling, while holding x.lock, a function whose summary takes x.lock again is a compile-time
# error". The first attack's "needs the lock" helper does not apply: `deposit` is also called from outside
# with no lock held, so it must take the lock itself.
from std.sync import Lock

class Account:
    lock: Lock = Lock()
    balance: int = 0

    fn deposit(self, n: int):
        with self.lock:
            self.balance += n

    fn deposit_all(self, amounts: Array[int]):
        with self.lock:
            for n in amounts:
                self.deposit(n)       # takes self.lock again
