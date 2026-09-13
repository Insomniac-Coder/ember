#$ test: run-pass
#$ rules: ARN-5, ARN-5a, ARN-5b, ARN-5d, ARN-5e, ARN-5f, ARN-5g, TST-24
#$ assert-c-count: contains("ember_arena_alloc_zeroed") == 1

from std.collections import ArenaMap, CapacityError

fn option_value(value: Option[ref i32]) -> i32:
    match value:
        Some(found):
            return found
        None:
            return -1

fn owned_value(value: Option[i32]) -> i32:
    match value:
        Some(found):
            return found
        None:
            return -1

fn inserted(value: Result[Option[i32], CapacityError]) -> i32:
    match value:
        Ok(previous):
            match previous:
                Some(found):
                    return found
                None:
                    return 0
        Err(_full):
            return -2

fn main():
    arena = Arena.with_capacity(4096)
    values: ArenaMap[i32, i32] = ArenaMap[i32, i32].with_capacity(arena, 2)
    println(values.len())
    println(values.capacity())
    println(values.is_empty())
    println(inserted(values.insert(1, 10)))
    println(inserted(values.insert(2, 20)))
    println(inserted(values.insert(3, 30)))
    println(inserted(values.insert(1, 11)))
    println(values.len())
    println(option_value(values.get(1)))
    println(option_value(values.get(2)))
    println(values.contains_key(1))
    println(values.contains_key(3))
    match values.get_mut(2):
        Some(found):
            replacement: i32 = found + 1
            found = ref mut replacement
        None:
            pass
    total: i32 = 0
    for pair in values.iter():
        total = total + pair.0 + pair.1
    println(total)
    println(owned_value(values.remove(1)))
    println(option_value(values.get(1)))
    println(option_value(values.get(2)))
    values.clear()
    println(values.len())
    println(values.is_empty())
#$ stdout: 0
#$ 2
#$ true
#$ 0
#$ 0
#$ -2
#$ 10
#$ 2
#$ 11
#$ 20
#$ true
#$ false
#$ 35
#$ 11
#$ -1
#$ 21
#$ 0
#$ true
