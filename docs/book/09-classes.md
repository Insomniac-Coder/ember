# 9. Classes

A `struct` is a value with one owner. A `class` is an **object**: several names can refer to the
same one, through **handles**, and it lives as long as any handle to it does. Use a class for a
thing with an identity — a player, a window, a node in a graph — and a struct for plain data.

```ember
class Counter:
    name: String
    count: int = 0

    fn bump():
        this.count += 1

fn main():
    a = Counter("clicks")
    b = a
    b.bump()
    a.bump()
    println(a.name, a.count, b.count)
```

```output
clicks 2 2
```

`b = a` copies the handle, not the object: `a` and `b` are one counter. The object is freed when
its last handle goes, at that point in the program — not later, by a collector.

## Constructors

A class with an `init` method is made by calling it; one without is made from its fields, as a
struct is:

```ember
class Account:
    owner: String
    balance: int

    fn init(owner: StrView, deposit: int):
        this.owner = owner.to_string()
        this.balance = deposit

    fn withdraw(amount: int) -> bool:
        if amount > this.balance:
            return false
        this.balance -= amount
        return true

fn main():
    account = Account("Ada", 100)
    println(account.withdraw(30), account.withdraw(500), account.balance)
```

```output
true false 70
```

## Inheritance

A class is closed to subclasses unless it says `open`. A subclass names its base in brackets, and a
method the base marks `virtual` is replaced by an `override`:

```ember
open class Animal:
    name: String

    virtual fn sound() -> String:
        return "..."

    fn speak() -> String:
        return f"{this.name} says {this.sound()}"

class Dog(Animal):
    override fn sound() -> String:
        return "woof"

fn main():
    animals: Array[Animal] = [Animal("generic"), Dog("Rex")]
    for animal in animals:
        println(animal.speak())
```

```output
generic says ...
Rex says woof
```

## Cycles and `Weak`

Two objects holding handles to each other keep each other alive: neither is freed. A debug run
reports such a cycle when the program ends, naming the field to change. A back-pointer — a child to
its parent, an item to its list — is a `Weak` handle, which does not keep its object alive;
`upgrade()` gives the object back as an `Option`, `None` once it is gone:

```ember
class Parent:
    name: String
    children: Array[Child] = []

class Child:
    name: String
    parent: Option[Weak[Parent]] = None

fn main():
    mum = Parent("Ada")
    kid = Child("Byron")
    kid.parent = Some(Weak(mum))
    mum.children.push(kid)
    for child in mum.children:
        if child.parent exists as weak:
            if weak.upgrade() exists as parent:
                println(f"{child.name} is {parent.name}'s")
```

```output
Byron is Ada's
```
