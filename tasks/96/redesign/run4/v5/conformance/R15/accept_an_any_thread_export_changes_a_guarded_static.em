#$ test: compile-pass
#$ rules: R15
## R15: the lines E7012's help gives: `HITS` guarded by a `static` lock, the export's body inside it
## (AN-84's NEW).

from std.sync import Lock

static HITS_LOCK: Lock = Lock()
static mut HITS: int guarded by HITS_LOCK = 0

@export(threads=any)
fn bump():
    with HITS_LOCK:
        HITS += 1

fn main():
    with HITS_LOCK:
        println(HITS)
