#$ test: run-pass
#$ rules: CLS-4, CLS-7, EXC-1, EXP-1
#$ profiles: debug, release, shipping
#$ assert-c: contains(em_Child_bump)
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 1
#$ assert-c: contains(ember_object_begin_write)
#$ assert-c: contains(ember_object_end_write)
#$ stdout: 7

class Child:
    value: i32

    fn bump(mut self, amount: i32):
        self.value = self.value + amount

fn main():
    child = Child(2)
    items: Array[Child] = Array[Child]()
    items.push(child)
    items[0].bump(5)
    println(items[0].value)
