#$ test: run-pass
#$ rules: CLO-1, CLO-2, CLO-3, CLO-6, OWN-3, OWN-5, TST-19
#$ stdout: 1
#$ assert-c-count: contains("ember_vec_free(") == 1

# Mutation alone is reusable, but this body subsequently moves its Array into
# `consume`; the resulting closure is therefore one-shot and consumes its
# environment at the call.

fn consume(owned values: Array[i32]) -> usize:
    return values.len()

fn main():
    values: Array[i32] = Array[i32]()
    task = owned fn() -> usize:
        values.push(7)
        return consume(values)
    println(task())
