# K70-raw-identifier-reserved

Question: Can a C field named owned or self be reached as r#owned?
Severity: low; impact rank 70
Members: J1-14; primary J1-14
Why together: Single question.
Critic's notes: 

## Finding J1-14
Rules: LEX-14, LEX-15, FFI identifier table; file tasks/spec-0.9.9/parts/p02-lexical.md lines 71-74, 109-113; kind undecided_case; severity low
Quote from Hardened_11:
(lexed as keywords so that no program uses them as names; any use is `E1010`,

The finder's question:
Can a C struct field or function named `owned`, `self`, `async`, `await` or `macro` be imported and used as
`r#owned` or `r#self`? LEX-14 lets `r#name` use 'a keyword' and the FFI table maps 'an identifier that is an
Ember keyword' to a raw identifier, but these words are listed apart from the 52 keywords and 'any use' of
`owned` and `self` is E1010.

Program:
```
# buffer.h:  struct buffer { char *data; size_t len; bool owned; };
import c "buffer.h"

fn release(ref mut b: buffer.buffer):
    if b.r#owned:
        free_data(ref mut b)
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.5)
Reason:
LEX-14 (p02 71-72) says `r#name` 'uses a keyword as a name', and the FFI table (p16 194) maps 'an identifier
that is an Ember keyword' to `r#`. LEX-15 (p02 109-113) says `async`, `await`, `macro`, `owned` and `self` are
'lexed as keywords' but stand outside 'the complete reserved set' of 52 (p02 90), and 'any use' of
`owned`/`self` is E1010. Whether `r#owned` or `r#self` is a use of the reserved word, or a raw identifier that
lifts it, is not said. C headers do use `owned` and `self` as field names. The literal reading leans toward
`r#` working, but the text supports both. Thin evidence, hence low confidence.

options:
[{'letter': 'a', 'summary': '`r#` names any word Ember reads as a keyword, the reserved ones included',
'code': 'if b.r#owned:\n    println("the buffer frees its own data")', 'outcome': "Compiles, and the importer
names the C field `r#owned`. The bare word keeps its error: `owned` alone is E1010 ('write `owns`'), and
`async` alone is E0005."}, {'letter': 'b', 'summary': '`r#` only for the 52 keywords; the reserved words
cannot be names at all', 'code': '// buffer_shim.c: bool buffer_is_owned(const struct buffer *b) { return
b->owned; }\nif buf.buffer_is_owned(b):\n    println("the buffer frees its own data")', 'outcome':
'`b.r#owned` is E1010. The C field is reachable only through a C function written for it.'}]

recommendation:
{'letter': 'a', 'reason': '`r#` exists so that C names Ember reserves can still be reached. A field named
`owned` or `self` in a C struct is no different from one named `type`, and refusing it would push a correct
program into writing C.', 'checklist': '2, 5 (one rule for every word the lexer reserves), 7'}

