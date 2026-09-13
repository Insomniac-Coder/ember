#$ test: run-pass
#$ rules: ARN-5, ARN-5a, ARN-5b, ARN-5c, ARN-5e, ARN-5f, ARN-5g, TST-24
#$ assert-c-count: contains("ember_arena_alloc_uninit") == 1

from std.collections import ArenaArray, CapacityError

fn show(value: Option[ref i32]) -> i32:
    match value:
        Some(found):
            return found
        None:
            return -1

fn status(value: Result[void, CapacityError]) -> i32:
    match value:
        Ok(_unit):
            return 1
        Err(_full):
            return 0

fn removed(value: Option[i32]) -> i32:
    match value:
        Some(found):
            return found
        None:
            return -1

fn main():
    arena = Arena.with_capacity(4096)
    values: ArenaArray[i32] = ArenaArray[i32].with_capacity(arena, 3)
    println(values.len())
    println(values.capacity())
    println(values.is_empty())
    println(status(values.push(10)))
    println(status(values.push(30)))
    println(status(values.insert(1, 20)))
    println(status(values.push(40)))
    println(show(values.get(0)))
    println(show(values.get(1)))
    println(show(values.get(2)))
    println(show(values.get(3)))
    total: i32 = 0
    for item in values.iter():
        total = total + item
    println(total)
    for item in values.iter_mut():
        replacement: i32 = item + 1
        item = ref mut replacement
    println(show(values.get(0)))
    println(show(values.get(2)))
    match values.get_mut(0):
        Some(item):
            replacement: i32 = item + 100
            item = ref mut replacement
        None:
            pass
    println(show(values.get(0)))
    println(removed(values.remove(1)))
    println(show(values.get(1)))
    values.clear()
    println(values.len())
    println(values.is_empty())
#$ stdout: 0
#$ 3
#$ true
#$ 1
#$ 1
#$ 1
#$ 0
#$ 10
#$ 20
#$ 30
#$ -1
#$ 60
#$ 11
#$ 31
#$ 111
#$ 21
#$ 31
#$ 0
#$ true
