# K26-copying-a-view

Question: What does copying a view give: w = this.rest of a StrView field, and copy of a StrView?
Severity: medium; impact rank 26
Members: J4-08, J2-01; primary J4-08
Why together: J4-08 asks what = gives from a view field, and J2-01 what copy of gives before a view. J4-08 B's outcome ('copy of a StrView is refused too') holds only under J2-01 C, so each answer changes the other's options.
Critic's notes: Ordinary lexer code. J4-02's refutation (copy of a StrView is settled as E2116) does not hold against J2-01 (see conflicts). Ask before K62 (J1-10).

## Finding J4-08
Rules: BRW-12, BRW-1, LT-20, LT-1a; file tasks/spec-0.9.9/parts/p07-ownership.md lines 194-207; kind undecided_case; severity medium
Quote from Hardened_11:
is a read-only view of it: never a hand-over,
  and never a silent copy, except that a part with no fields of its own (a number, a `bool`, a variant
  with no payload) is copied, as Python binds a number.

The finder's question:
In a lexer, is `w = this.rest` (with `rest: StrView`) a view of the field `this.rest`, so that `this.rest =
w[i + 1..]` while `w` is still used is E3021? Or is it a copy of the `StrView`, so the method compiles?

Program:
```
struct Lexer:
    rest: StrView

    fn next_word() -> Option[StrView]:
        w = this.rest
        if w.find(" ") exists as i:
            this.rest = w[i + 1..]     # E3021 if `w` views the field; fine if `w` is a copy of the view
            return Some(w[..i])
        this.rest = ""
        return Some(w)
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.5)
Reason:
Not settled. [BRW-12] copies only "a part with no fields of its own (a number, a `bool`, a variant with no
payload)" and makes every other part a read-only view of the part itself. Whether a `StrView`, `Span` or `ref
T` field counts as having no fields of its own is not said. [OWN-9] copies a whole `StrView` name with `=`,
and [LT-20] carries only the field's region on a projection, but neither says whether the new name views the
field's place. The compiler's own conformance test
LT-22/accept_items_of_a_non_copy_iterator_point_into_what_it_reads accepts exactly this pattern (`rest =
this.rest`, then `this.rest = rest[at + 1..]` while `rest` is used). That is the lead's reading, not a rule,
and the owner's N26 answer (verdict 137) concerned where the result points, not this line.

options:
[{'letter': 'A', 'summary': 'A part whose type is a `Copy` view (`StrView`, `Span[T]`, `ref T`) is copied, as
`=` copies a whole view name: the new name looks at what the field looks at.', 'code': 'w = this.rest\nif
w.find(" ") exists as i:\n    this.rest = w[i + 1..]\n    return Some(w[..i])', 'outcome': "Compiles; prints
Some('let') Some('x'). Writing `this.rest` does not touch `w`, which looks at the text. A list element of a
view type (`w = words[0]`) is copied the same way."}, {'letter': 'B', 'summary': '`w` is a read-only view of
the field `this.rest` itself.', 'code': 'fn next_word() -> Option[String]:\n    w = this.rest\n    if w.find("
") exists as i:\n        word = String(w[..i])\n        this.rest = w[i + 1..]\n        return Some(word)\n
...', 'outcome': 'The lexer as written is refused at `this.rest = w[i + 1..]` (E3021: it changes the field `w`
views while `w` is used on the next line). Since `copy of` a `StrView` is refused too, the method must copy
each word into a `String` and return `Option[String]`.'}]

recommendation:
{'letter': 'A', 'reason': "`=` already copies a whole `StrView`, `Span` or `ref` name, and copying one copies
no data, so the copy is no silent copy. The lexer then reads as it does in C++ (`std::string_view w = rest;`)
and in Rust (`let w = self.rest;`), and the compiler's own test already accepts it.", 'checklist': '1, 2
(accepts the correct lexer), 5 (one meaning of `=` for a view, whole name or field), 6 (nearest rule: `=`
copies a view name), 10'}

## Finding J2-01
Rules: TYP-36, OWN-8, STR-3, OWN-7; file tasks/spec-0.9.9/parts/p04-types.md lines 576, 693-695, 710; kind contradiction; severity medium
Quote from Hardened_11:
`copy of x` makes a value with `Clone` (`[OWN-8]`), and every `Copy` type is `Clone`; before what it cannot
copy as itself (a `weakref` link, a value of a view type, a type with no `Clone`) it is `E2116`.

The finder's question:
Does `copy of` compile for a `StrView`, a `Span[T]`, or a `@derive(Copy)` struct holding a `StrView`? If it
does, is the result a view (Clone of StrView returns Self) or text of your own?

Program:
```
@derive(Copy)
struct Token:
    text: StrView
    line: int

fn main():
    src = String("let x = 1")
    tok = Token(text=src.as_str(), line=1)
    t2 = copy of tok          # STR-3 / verdict 68: works on every Copy struct; OWN-8: E2116, a view type
    w = src.as_str()
    w2 = copy of w            # TYP-36 table: StrView is Clone and Copy; OWN-8: E2116 "to make your own copy write String(w)"
    println(t2.line, w2)
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.85)
Reason:
OWN-8 (p07 167-171) says every Copy type has Clone 'so b = copy of a works', and verdict 68 says a Copy type
always works with copy of, E2116 never given for it. The same rule (p07 182-183), TYP-36 (p04 694-695) and the
E2116 row (p17a 85) then name StrView and Span[T], both always Copy by the marker table (p04 576), as E2116. A
@derive(Copy) struct holding a StrView is a Copy struct (STR-3 says copy of works on every one) and a view
type (TYP-36 refuses it). No verdict says what copy of gives before a view: verdict 10 says the copy word
copies the thing itself, never a link; verdict 79 says a link inside a struct is copied as a link; verdict 93
has copy of t on a loop view of a String give a copy of the text. The rules leave the result of copy of w
open: refused, a String, or another view.

options:
[{'letter': 'A', 'summary': '`copy of` before a view copies what it views (a StrView gives a String, a Span[T]
an Array[T], a `ref T` a T); a view inside a struct is copied as one more view of the same place, as a link
inside a struct is copied as one more link.', 'code': 't2 = copy of tok     # a new Token whose `text` views
the same source text\nw2 = copy of w       # a String of its own, as String(w) makes', 'outcome': 'Compiles.
Prints `let x = 1 1`, then `let x = 1`: `w2` is its own text, untouched by `src.clear()`. `copy of` before a
`ref mut` view or a `MutSpan`, or a struct holding one, stays E2116 (no second changing view can be made).'},
{'letter': 'B', 'summary': 'A Copy view is copied bit for bit, as `=` copies it: `copy of w` is another
StrView of the same text and `copy of tok` another Token; only views that are not Copy (`ref mut`, `MutSpan`)
are E2116.', 'code': "t2 = copy of tok     # a Token, the same bits\nw2 = copy of w       # another StrView of
`src`'s text", 'outcome': 'Lines 9-12 compile and print `let x = 1 1`; `println(w2)` after `src.clear()` is
refused (E3060: `w2` views `src`, which `clear` reshaped). For text of your own the programmer writes
`String(w)`.'}, {'letter': 'C', 'summary': 'As the refusal list says today: `copy of` before a StrView, a Span
or any struct holding a view is E2116, Copy or not; the general sentence is narrowed to say so.', 'code': 't2
= tok             # `=` copies a Copy struct\nw2 = String(w)       # text of your own', 'outcome': '`copy of
tok` and `copy of w` are E2116 ("`w` is a view of `src`\'s text; to make your own copy write `String(w)`");
with the lines above the program compiles and prints the same as A.'}]

recommendation:
{'letter': 'A', 'reason': 'It is the one reading that keeps every decision at once: the copy word copies the
thing itself, never a link, before a bare link or view; a link or view inside a struct is copied as one more
link; and every Copy type works with `copy of`. Nothing is refused that has a meaning, and the result is never
a hidden second view.', 'checklist': '1 (copy of means a copy of your own), 2 (correct programs accepted), 5
(copy of means one thing for links and views alike), 6 (the same rule as shares/sharedref mut links, bare and
in fields), 11 (reuses the struct-field link rule already written)'}

