#$ test: run-pass
#$ rules: STD-15, BRW-5, SPN-1, GRM-34
#$ stdout:
#$ true 3
#$ [5, 1, 4, 7, 8]
#$ 5 [8, 1, 4, 7]
#$ [1, 8, 4, 7]
#$ [1, 8] [1, 8]
#$ [1, 18] [1] [18] 19 [2, 36]
#$ ['a', 'b', 'c']
#$ [3, 1, 1, 5, 9] [1, 2, 3, 1]
#$ Ok(2) Err(2) Err(0) Err(4)
# `[STD-15]`'s `Array` methods: `capacity`, `reserve`, `extend`,
# `swap_remove`, `swap` (`[BRW-5]`), `truncate`, `get_mut`, `split_at`,
# `iter`, `iter_mut`, and, written in Ember in `std.core` as a generic
# extension (`[GRM-34]`), `retain`, `dedup` and `binary_search`.

xs = [5, 1, 4]
xs.reserve(10)
println(xs.capacity() >= 13, len(xs))
xs.extend([7, 8])
println(xs)
println(xs.swap_remove(0), xs)
xs.swap(0, 1)
println(xs)
xs.truncate(2)
kept = xs.clone()
xs.truncate(9)
println(kept, xs)
match xs.get_mut(1):
    Some(r): r += 10
    None: pass
left, right = xs.split_at(1)
total = 0
for x in xs.iter():
    total += x
print(xs, left, right, total, "")
for x in xs.iter_mut():
    x *= 2
println(xs)
names = [String.from("a")]
names.extend([String.from("b"), String.from("c")])
names.swap(0, 2)
first = names.swap_remove(0)
names.truncate(9)
names.push(first)
println(names)
odd = [3, 1, 4, 1, 5, 9, 2, 6]
odd.retain(fn(x) => x % 2 == 1)
runs = [1, 1, 2, 2, 2, 3, 1]
runs.dedup()
println(odd, runs)
sorted_ = [1, 3, 5, 7]
println(sorted_.binary_search(5), sorted_.binary_search(4), sorted_.binary_search(0), sorted_.binary_search(9))
