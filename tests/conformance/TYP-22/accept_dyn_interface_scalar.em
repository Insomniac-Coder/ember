#$ test: run-pass
#$ rules: TYP-22, IFC-1, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 41
#$ stdout: 41
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Inspect em_vt_dyn_Inspect_i32)
#$ assert-c: contains(em_vt_dyn_Inspect_i32_drop)
#$ assert-c: contains(return em_i32_inspect(*(int32_t*)_0);)
#$ assert-c: contains(ember_box_new_copy(sizeof(int32_t), _Alignof(int32_t), &((int32_t){42}))

interface Inspect:
    fn inspect(self) -> i32

extend i32 implements Inspect:
    fn inspect(self) -> i32:
        return self

fn inspect_ref(value: ref dyn Inspect) -> i32:
    return value.inspect()

fn main():
    value: i32 = 41
    println(inspect_ref(ref value))
    boxed: Box[dyn Inspect] = Box(value)
    println(boxed.inspect())
    boxed_literal: Box[dyn Inspect] = Box(42)
    println(boxed_literal.inspect())
