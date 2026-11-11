#$ test: run-pass
#$ rules: TYP-22, IFC-1, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_5Truth em_vt_dyn_5Truth_for_3std4core4bool)
#$ assert-c: contains(em_vt_dyn_5Truth_for_3std4core4bool_drop)
#$ assert-c: contains(ember_box_new_copy(sizeof(bool), _Alignof(bool), &((bool){true}))

interface Truth:
    fn truth() -> bool

extend bool implements Truth:
    fn truth() -> bool:
        return this

fn main():
    boxed: owns dyn Truth = owns true
    if boxed.truth():
        println(42)
