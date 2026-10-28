#$ test: compile-pass
#$ rules: R9, R15
## R9: the nearest allowed program: the cache is `thread_shared`, whose count is atomic, so an owner of
## it may go to another thread (AN-71's message, applied).

import std.thread

class Cache:
    hits: int = 0

class Names:
    items: Array[String] = []
    cache: thread_shared Cache

fn share_cached():
    c = thread_shared Cache()
    names = Names(cache=thread_shared c)
    h = thread.spawn(fn() with owns names: names.items.len())
    println(c.hits, h.join())

fn main():
    share_cached()
