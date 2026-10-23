# A5 (ii): NLL problem case #3 in Ember (asset cache).
from std.collections import Map

fn get_or_load(mut cache: Map[int, String], id: int) -> ref String:
    match cache.get(id):
        Some(s):
            return s
        None:
            pass
    cache[id] = f"asset {id}"
    return ref cache[id]

fn main():
    cache = Map[int, String]()
    println(get_or_load(cache, 3))
