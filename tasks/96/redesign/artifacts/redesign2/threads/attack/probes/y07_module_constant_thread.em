# PROPOSED syntax. A module variable never assigned after its initialiser, read by an unscoped thread:
# refused by 6.3 rule 5. Today's spelling with `static` compiles (x03_static_read_thread.em, exit 0).
import std.thread

DEBUG = true

fn main():
    worker = thread.spawn(fn(): println(DEBUG))
    worker.join()
