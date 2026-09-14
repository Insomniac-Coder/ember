#$ test: compile-pass
#$ rules: CLS-1, CLS-3, OWN-2
#$ assert-c: contains(ember_obj_new(&em_ti_Empty))

class Empty:
    fn answer(self) -> i32:
        return 42

fn main():
    value = Empty()
    println(value.answer())
