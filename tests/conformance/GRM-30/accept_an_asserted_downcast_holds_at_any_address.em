#$ test: run-pass
#$ rules: GRM-30, TYP-6, CLI-21
#$ profiles: debug, development, release
#$ stdout: 300 44850 100
## `[GRM-30]` (D-899) — `cast a as Dog` holds for every `Dog`, wherever its object is: the check is
## that the handle is not null. Three hundred objects kept alive, boxes made between some of them,
## lie at as many addresses; the evaluator read only the handle's low byte, so one at an address
## ending in a zero byte panicked "invalid downcast" there (and the compiled program ran on).

open class Animal:
    fn name(self) -> int:
        return 0

class Dog(Animal):
    tag: int = 0

    fn bark(self) -> int:
        return self.tag

fn main():
    animals: Array[Animal] = []
    boxes: Array[Box[int]] = []
    for i in 0..300:
        d = Dog()
        d.tag = i
        animals.push(d)
        if i % 3 == 0:
            boxes.push(Box(i))
    count = 0
    total = 0
    for a in animals:
        dog = cast a as Dog
        count += 1
        total += dog.bark()
    println(count, total, boxes.len())
