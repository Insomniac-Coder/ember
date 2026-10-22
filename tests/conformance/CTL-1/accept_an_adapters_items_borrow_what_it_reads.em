#$ test: run-pass
#$ rules: CTL-1, LT-1a, STD-19, BRW-2
#$ profiles: debug, development, release
#$ stdout: bbb
#$ ccc dd
#$ (2, 'dd') ('dd', 'a')
#$ bbb dd
#$ Some('dd') 3
## `[CTL-1]`, `[LT-1a]` (ODR-211) — an iterator adapter's items borrow what the iterator it wraps
## reads, not the adapter: `Filter`, `Map`, `Take`, `Skip`, `Enumerate`, `Zip`, `Chain`, `Rev`,
## `StepBy`, `Inspect`, `TakeWhile`, `SkipWhile` declare `@borrows(self.inner)` (or their two
## sources) on `next`, a field whose type is a type parameter, and each instance applies it where
## the items are views. So the "best so far" idiom keeps an item of `m.keys().filter(f)` across
## turns and after the loop (it was `E3025` and `E3060` naming `__it`, D-856), and an item
## outlives the adapter it came from. A consumer that gives an item (`find`) declares
## `@borrows(self)`: the item is the iterator's, not the predicate's, so a predicate that counts
## what it sees leaves its counter readable while the item lives.

fn keep_last[I: Iterator](owned it: I) -> Option[I.Item]:
    last: Option[I.Item] = None
    for x in it:
        last = Some(x)
    return last

fn main():
    m: Map[String, int] = {}
    m["a".to_string()] = 1
    m["bbb".to_string()] = 2
    m["dd".to_string()] = 3
    best: Option[ref String] = None
    for k in m.keys().filter(fn(k: ref String) => k.len() > 1):
        match best:
            Some(b):
                if k.len() > b.len():
                    best = Some(k)
            None:
                best = Some(k)
    println(best.unwrap())
    words = ["a".to_string(), "ccc".to_string(), "dd".to_string()]
    longest: Option[ref String] = None
    shortest: Option[ref String] = None
    for w in words.iter().map(fn(w: ref String) -> ref String => w).skip(1).take(2):
        if longest is None:
            longest = Some(w)
        shortest = Some(w)
    println(longest.unwrap(), shortest.unwrap())
    first_pair: Option[(int, ref String)] = None
    for p in words.iter().enumerate().rev().take(2):
        if first_pair is None:
            first_pair = Some(p)
    zipped: Option[(ref String, ref String)] = None
    for z in words.iter().zip(words.iter().rev()):
        zipped = Some(z)
    println(first_pair.unwrap(), zipped.unwrap())
    stepped = keep_last(words.iter().chain(m.keys()).step_by(2).inspect(fn(w: ref String): pass))
    tail = keep_last(words.iter().take_while(fn(w: ref String) => w.len() < 4).skip_while(fn(w: ref String) => w.len() < 2))
    println(stepped.unwrap(), tail.unwrap())
    looked = 0
    two = fn(w: ref String) -> bool:
        looked += 1
        return w.len() == 2
    found = words.iter().find(two)
    println(found, looked)
