#$ test: compile-pass
#$ rules: TYP-22, FN-2, FN-2a
#$ assert-c: contains(int32_t (*slot0)(void*, int32_t, int32_t*))

interface Adjust:
    fn adjust(self, amount: i32, mut value: i32) -> i32

fn take(x: ref dyn Adjust) -> i32:
    value: i32 = 1
    return x.adjust(value = value, amount = 2)

fn main():
    pass
