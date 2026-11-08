# K19-written-init-call-words

Question: Does a call of a class's own init say owns before an owns parameter's argument?
Severity: high; impact rank 19
Members: J1-07; primary J1-07
Why together: Single question.
Critic's notes: Every class with an init meets this. The draft must quote verdict 159 ('A constructor's argument takes no word: it works as `=` does') and say that A narrows it to constructors made from the fields (verdict clash). CLS-2's E2108 help and I.5's `Emitter("fountain")` depend on the answer.

## Finding J1-07
Rules: FN-2a, OWN-9, GRM-15, CLS-2, STR-6; file tasks/spec-0.9.9/parts/p01-overview.md lines 176-177, 195, 203-204; kind undecided_case; severity high
Quote from Hardened_11:
*Note.* `Emitter("fountain")` passes a string literal where a `String` is expected; the literal
allocates the `String` there (`[TXT-9]`), and `owns name` takes it over.

The finder's question:
For a class whose written `init` has an `owns` parameter, is the call `Emitter(label)` or `Emitter(owns
label)`? And for an `init` with a plain parameter (`fn init(name: String)`, which E2108's help suggests), does
`Emitter(label)` hand `label` over, as `=` does, or only lend it?

Program:
```
class Emitter:
    name: String
    fn init(owns name: String):
        this.name = name

fn main():
    label = String("fountain")
    a = Emitter(label)          # FN-2a/OWN-9: a constructor's argument takes no word
    b = Emitter(owns label)     # GRM-15/FN-2a: an `owns` parameter says `owns` at the call
```

The checker's verdict: confirmed (route owner_odr, severity high, confidence 0.75)
Reason:
Verdict 159 was decided on a struct's memberwise constructor (`Words(names=xs)`): 'A constructor's argument
takes no word: it works as `=` does (verdict 147 does not reach a constructor)'. FN-2a (p05 131-132) and OWN-9
(p07 93-94) say it of every constructor. CLS-2 (p05 313) and STR-6 (p05 253) call a written `fn init(...)`
'the constructor'. But a written init has a first line with its own parameter words, and FN-2a and GRM-15
require `owns` before an argument for an `owns` parameter. Neither rule says which wins for `Emitter(...)`
when `init(owns name: String)` exists. With a plain init parameter, the 'as `=`' reading would hand over a
value the init only reads, against FN-2 (p05 112: 'A parameter with no word reads'). I.5 (p01 195) writes
`Emitter("fountain")` for `init(owns name: String)`, so it depends on the answer.

options:
[{'letter': 'a', 'summary': "A written init is called like any function, with its parameters' words", 'code':
'fountain = Emitter(owns title)       # init says `owns name`\nsign = Label(caption)                # init
only reads `text`\nw = Words(names=xs)                  # a struct with no init: as `=`, no word', 'outcome':
'Compiles. `title` ends at the call, and `caption` stays the program\'s, so `println(caption)` prints smoke.
`Emitter(title)` is E3028 (\'say so at the call: `Emitter(owns title)`\'). The overview\'s line becomes
`Emitter(owns "fountain")`. A constructor made from the fields with no init keeps no word, as decided.'},
{'letter': 'b', 'summary': "No owns at any constructor call; the init's parameter word decides what happens",
'code': 'fountain = Emitter(title)            # handed over: init says `owns name`\nsign = Label(caption)
# lent: init only reads `text`', 'outcome': "Compiles. `title` ends at the call and `caption` stays, so
`println(caption)` prints smoke. `Emitter(owns title)` is refused ('remove `owns`'). A `ref mut` parameter of
an init is still written `ref mut` at the call, as at every call."}, {'letter': 'c', 'summary': "Every
constructor argument works as `=`, whatever the init's first line says", 'code': 'fountain =
Emitter(title)\nsign = Label(caption)', 'outcome': "`title` and `caption` both end at their calls, although
`Label`'s init only reads `text`, so `println(caption)` is E3040 ('`caption` was handed to `Label`')."}]

recommendation:
{'letter': 'a', 'reason': "A written init has a first line with words, like any function, and the rule that
the call repeats the parameter's word (`take(owns xs)`) was chosen so that each line shows what happens to
each name. A constructor made from the fields has no first line to repeat, which is why its argument works as
`=`.", 'checklist': '5 (one word, one meaning at every call), 6 (the exception holds only where there is no
first line to repeat), 11, 15, 2'}

