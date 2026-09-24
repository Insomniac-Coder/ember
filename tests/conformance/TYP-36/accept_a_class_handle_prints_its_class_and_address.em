#$ test: run-pass
#$ rules: TYP-36, STD-9
#$ profiles: debug, release, shipping
#$ stdout: <Token at <Token at
#$ <Dog at
#$ <Token at
#$ [<Token at
# `[TYP-36]` — a class handle has no `Display` but has `Debug`: its class and
# address. Printing falls back to it (`[STD-9]`), and the class is the
# object's own, whatever the handle's static type. The address differs from
# run to run, so only the text before it is checked.

open class Animal:
    legs: int

    fn init(mut self):
        self.legs = 4

class Dog(Animal):
    fn init(mut self):
        super.init()

interface Named:
    fn label(self) -> int

class Token implements Named:
    id: int

    fn label(self) -> int:
        return self.id

fn main():
    token = Token(id=1)
    println(f"{token}"[0..9], f"{token!r}"[0..9])
    pet: Animal = Dog()
    println(f"{pet}"[0..7])
    named: Named = token
    println(f"{named}"[0..9])
    println(f"{[token]}"[0..10])
