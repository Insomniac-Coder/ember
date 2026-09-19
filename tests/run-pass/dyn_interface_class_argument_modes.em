#$ test: run-pass
#$ rules: TYP-22, IFC-1, FN-2, FN-2a
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(int32_t (*slot0)(void*, int32_t, int32_t*))
#$ assert-c: contains(return em_Adjuster_adjust((struct em_obj_Adjuster*)_0, _1, _2);)

interface Adjust:
    fn adjust(self, amount: i32, mut value: i32) -> i32

class Adjuster implements Adjust:
    fn adjust(self, amount: i32, mut value: i32) -> i32:
        value = value + amount
        return value

fn adjust_it(source: ref dyn Adjust, mut value: i32) -> i32:
    return source.adjust(value = value, amount = 2)

fn main():
    adjuster = Adjuster()
    value: i32 = 40
    println(adjust_it(ref adjuster, value = value))
