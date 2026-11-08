# K32-super-init-memberwise

Question: When a base class has no init, what does super.init(…) call?
Severity: medium; impact rank 32
Members: J3-05; primary J3-05
Why together: Single question.
Critic's notes: Under every K19 option, a memberwise base's `super.init(id=id)` takes no words, so the dependency on K19 is only wording.

## Finding J3-05
Rules: CLS-3, CLS-4, CLS-10, CLS-11; file tasks/spec-0.9.9/parts/p05-declarations.md lines 331-334; kind undecided_case; severity medium
Quote from Hardened_11:
* `[CLS-3]` A class with no `init` and no base class gets a memberwise constructor as a struct does.
* `[CLS-10]` *(new in 0.9.9)* A derived class with no `init` gets its base's constructor: `Door(entity)`
  evaluates `Door`'s field defaults and then runs `Script.init(entity)`.

The finder's question:
A base class has no `init`, so it has a memberwise constructor. What does a derived class's `super.init(…)`
call, and does a derived class with no `init` get the base's memberwise constructor?

Program:
```
open class Widget:
    name: String
    visible: bool = true

class Button(Widget):
    label: String

    fn init(name: String, label: String):
        this.label = label
        super.init(name=name)

fn main():
    b = Button(name=String("ok"), label=String("OK"))
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.7)
Reason:
CLS-3 (p05:331) gives a memberwise constructor only to "a class with no `init` and no base class". CLS-10
(p05:332-334) and its example assume the base declares `init`. CLS-4 (p05:344-345) says "A derived `init` MUST
call `super.init(…)` exactly once". Nothing says what `super.init(…)` calls when the base has no `init`, or
whether it takes named arguments and leaves out defaulted fields. A grep of `super.init` finds only CLS-4,
CLS-11 and the C++-base rules (p21:124-129).

options:
[{'letter': 'A', 'summary': 'A class with no `init` has its memberwise constructor as its `init`, and
`super.init(…)` calls it like any base `init`.', 'code': "    fn init(id: int, width: int):\n
this.width = width\n        super.init(id=id)               # Widget's memberwise constructor; `visible` keeps
its default", 'outcome': "Compiles and prints `7 true 120`. Arguments may be positional or named, and fields
with defaults may be left out, as in `Widget(id=7)`. A derived class with no `init` whose base has none gets
the base's memberwise constructor too: `Panel(id=3)`."}, {'letter': 'B', 'summary': 'A base class that has a
field with no default must declare `init` before a class may derive from it.', 'code': 'open class Widget:\n
id: int\n    visible: bool = true\n\n    fn init(id: int):                   # now required\n        this.id =
id', 'outcome': 'The program as written is refused at `class Button(Widget):`, and the help gives Widget the
`init` above. A base whose fields all have defaults is called `super.init()`, with no arguments.'}]

recommendation:
{'letter': 'A', 'reason': 'The base already has a constructor, so `super.init(…)` calling it adds no special
case and no boilerplate `init`. It is the same constructor `Widget(id=7)` calls.', 'checklist': '2 (accepts a
correct program); 6 and 11 (reuses the memberwise constructor and its argument rules as they are); 12 (no
forced `init` only because the syntax to reach the base constructor is missing); 13 (no separate path for
bases without `init`).'}

