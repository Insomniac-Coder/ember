#$ test: compile-fail
#$ rules: CLO-1, CLO-2, CLO-3, CLO-6, OWN-3, TST-19

# Calling a concrete `CallableOnce` closure moves its environment at the
# first call. The ordinary moved-value diagnostic enforces the second call.

fn consume(owned values: Array[i32]) -> i32:
    return values[0]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    task = owned fn() => consume(values)
    _first = task()
    println(task()) #$ error[E3040]: `task` has been moved out of
