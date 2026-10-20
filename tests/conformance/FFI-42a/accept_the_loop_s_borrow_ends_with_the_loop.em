#$ test: run-pass
#$ rules: FFI-42a
#$ profiles: debug, development, release
#$ warning[W5002]: `Bag.begin` is not imported from `containers.hpp`: its result
#$ warning[W5002]: `Bag.end` is not imported from `containers.hpp`: its result
#$ stdout: 3 2
#$ stdout: 4 3
#$ stdout: 16
# `[FFI-42a]` — iterating an imported C++ container borrows it mutably for the loop only: inside
# it, the elements are read; after it, a method that grows the container (`put`, which could
# invalidate the loop's iterators) is called as ever, and the next loop walks the new elements.

import cpp "containers.hpp" with (overlay = "overlays/containers.em")

fn main():
    match cpp.Bag.new():
        Ok(bag):
            _ = bag.get_mut().put(1)
            _ = bag.get_mut().put(2)
            largest: i32 = 0
            for item in bag.get_mut():
                if item > largest:
                    largest = item
            _ = bag.get_mut().put(largest + 1)
            println(largest + 1, largest)
            count = 0
            for _ in bag.get_mut():
                count += 1
            _ = bag.get_mut().put(10)
            println(count + 1, count)
            total: i32 = 0
            for item in bag.get_mut():
                total += item
            println(total)
        Err(error):
            println(error.what)
