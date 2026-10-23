#$ test: run-pass
#$ rules: GRM-30, TYP-6
#$ profiles: debug, development, release
#$ stdout: 1 1 2
# ODR-280 — what `cast` reads is a name with its fields and indexes: `cast self.pet if Dog`,
# `cast animals[0] as Dog`, and `cast a if Dog` as a comprehension's element.

open class Animal:
    fn name(self) -> int:
        return 0

class Dog(Animal):
    fn bark(self) -> int:
        return 1

class Owner:
    pet: Animal

    fn init(mut self, pet: Animal):
        self.pet = pet

    fn bark(self) -> int:
        match cast self.pet if Dog:
            Some(dog):
                return dog.bark()
            None:
                return 0

fn main():
    animals: Array[Animal] = [Dog(), Dog()]
    owner = Owner(Dog())
    dogs = [cast a if Dog for a in animals]
    println(owner.bark(), (cast animals[0] as Dog).bark(), dogs.len())
