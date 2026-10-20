#$ test: run-pass
#$ rules: FFI-42, FFI-42a
#$ profiles: debug, development, release
#$ warning[W5002]: `Bag.begin` is not imported from `containers.hpp`: its result
#$ warning[W5002]: `Bag.end` is not imported from `containers.hpp`: its result
#$ stdout: 5 7 9
#$ stdout: 21 3
#$ stdout: 6 7 8
#$ stdout: 0
# `[FFI-42]` — a C++ class with `begin()` and `end()` is iterable: `for x in c` walks its C++
# iterators, a standard library's class (`std::vector<int>::iterator`) or a pointer, each element a
# `ref` to it, which reads through as the element. `[FFI-42a]` — the loop borrows the container
# mutably for its length: after it, the container's methods are called as ever. An empty container
# gives no element.

import cpp "containers.hpp" with (overlay = "overlays/containers.em")

fn main():
    match cpp.Bag.new():
        Ok(bag):
            for value in [5, 7, 9]:
                match bag.get_mut().put(value as i32):
                    Ok(_):
                        pass
                    Err(error):
                        println(error.what)
            seen: Array[i32] = []
            total: i32 = 0
            for item in bag.get_mut():
                seen.push(item)
                total += item
            println(seen[0], seen[1], seen[2])
            println(total, bag.get().size())
        Err(error):
            println(error.what)
    row = cpp.Row(cells = [5, 6, 7])
    after: Array[i32] = []
    for cell in row:
        after.push(cell + 1)
    println(after[0], after[1], after[2])
    match cpp.Bag.new():
        Ok(empty):
            count = 0
            for _ in empty.get_mut():
                count += 1
            println(count)
        Err(error):
            println(error.what)
