# A19: today a constant module-level value read from an unscoped thread is a static, and compiles?
import std.thread

static DEBUG: bool = true

fn main():
    h = thread.spawn(owned fn(): println(DEBUG))
    h.join()
