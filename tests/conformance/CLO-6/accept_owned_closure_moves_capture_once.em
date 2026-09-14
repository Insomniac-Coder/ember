#$ test: run-pass
#$ rules: CLO-1, CLO-2, CLO-3, CLO-6, OWN-3, OWN-5, TST-19
#$ stdout: 7
#$ assert-c-count: contains("ember_vec_free(") == 1

# An `owned fn` whose body consumes its Array capture is `CallableOnce`.
# Direct invocation moves the environment into `call_once`, so the Array has
# one owner and is destroyed exactly once.

fn consume(owned values: Array[i32]) -> i32:
    return values[0]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    task = owned fn() => consume(values)
    println(task())
