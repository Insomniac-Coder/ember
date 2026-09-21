#$ test: run-pass
#$ rules: TYP-22, IFC-1, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(static const struct em_vt_dyn_Truth em_vt_dyn_Truth_bool)
#$ assert-c: contains(em_vt_dyn_Truth_bool_drop)
#$ assert-c: contains(ember_box_new_copy(sizeof(bool), _Alignof(bool), &((bool){true}))

interface Truth:
    fn truth(self) -> bool

extend bool implements Truth:
    fn truth(self) -> bool:
        return self

fn main():
    boxed: Box[dyn Truth] = Box(true)
    if boxed.truth():
        println(42)
