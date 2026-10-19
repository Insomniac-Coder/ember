#$ test: run-pass
#$ rules: HND-1
#$ profiles: debug, release
#$ stdout: true 64
#$ stdout: false false
#$ stdout: true 8
#$ stdout: 9
# `[HND-1]` — a `Handle[T]` is a plain `Copy` value; every lookup compares its generation with the
# slot's, in every profile, so once its value is removed the handle (and each copy of it) finds
# nothing, even after the slot holds a new value. `get_unchecked` skips the comparison.

from std.collections import Pool

struct Texture:
    width: int
    height: int

fn main():
    pool = Pool[Texture]()
    h = pool.insert(Texture(width=64, height=64))
    copy = h
    println(pool.get(h).is_some(), pool[copy].width)
    pool.remove(h)
    fresh = pool.insert(Texture(width=8, height=8))
    println(pool.get(h).is_some(), pool.get(copy).is_some())
    println(pool.get(fresh).is_some(), pool[fresh].width)
    unsafe:  # SAFETY: `fresh` was just given out and nothing removed it
        println(pool.get_unchecked(fresh).width + 1)
