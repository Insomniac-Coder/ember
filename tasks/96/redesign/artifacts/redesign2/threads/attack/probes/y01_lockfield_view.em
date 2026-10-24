# PROPOSED syntax (phase3b-threads.md 4.3 to 4.5); not checkable by today's compiler.
# Rule 4 makes `p` an uncounted view of a parent object; rule 3 lets task 2 replace board.best under
# the lock; so "ann" can be freed while task 1 still prints p.name.
import std.thread
from std.sync import Lock

class Player:
    name: String = "ann".to_string()

class Board:
    lock: Lock = Lock()
    best: Player = Player()

fn run(board: Board):
    with scope = thread.scope():
        scope.spawn(fn():
            with board.lock:
                p = board.best              # a view: no count (rule 4)
            thread.sleep_ms(10)
            println(p.name))                # used after the lock
        scope.spawn(fn():
            with board.lock:
                board.best = Player(name="bob".to_string()))   # releases "ann", its last handle
