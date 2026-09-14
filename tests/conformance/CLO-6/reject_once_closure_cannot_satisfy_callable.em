#$ test: compile-fail
#$ rules: CLO-1, CLO-2, CLO-3, CLO-6, OWN-3, TST-19

# A plain `f: fn(...)` carries `Callable`, not `CallableOnce`; accepting this
# closure would promise two safe calls after the body has moved its Array.

fn consume(owned values: Array[i32]) -> i32:
    return values[0]

fn twice(f: fn() -> i32) -> i32:
    return f() + f()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    task = owned fn() => consume(values)
    println(twice(task)) #$ error[E3030]: closure would move a captured value out
