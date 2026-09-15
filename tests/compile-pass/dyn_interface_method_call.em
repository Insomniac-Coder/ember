#$ test: compile-pass
#$ rules: TYP-22, DSP-3
#$ assert-c: contains(struct em_vt_dyn_Drawable)
#$ assert-c: contains(->slot0)

interface Drawable:
    fn draw(self) -> i32

fn take(x: ref dyn Drawable) -> i32:
    return x.draw()

fn main():
    pass
