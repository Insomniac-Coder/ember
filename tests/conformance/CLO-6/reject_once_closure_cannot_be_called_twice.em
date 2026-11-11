#$ test: compile-fail
#$ rules: CLO-1, CLO-2, CLO-3, CLO-6, OWN-3, TST-19

# Calling a concrete `CallableOnce` closure moves its environment at the
# first call. The ordinary moved-value diagnostic enforces the second call.

fn consume(owns values: Array[i32]) -> i32:
    return values[0]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 7)
    task = owned fn() => consume(owns values)
    _first = task()
    println(task()) #$ error[E3040]: `task` was handed over to `em_closure_4main_c0` on line 14 and is used on line 15; to keep both, hand over a copy of your own: `copy of task`
