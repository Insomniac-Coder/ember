# K67-bool-char-casts

Question: May as convert a bool or a char to any integer type?
Severity: low; impact rank 67
Members: J2-14; primary J2-14
Why together: Single question.
Critic's notes: 

## Finding J2-14
The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.55)
Reason:
TYP-6 (p04 103-109) lists the explicit conversions: integer and float ones, and of `char` only `char → u32`
and `u8 → char`; `bool` has none. TYP-5 (p04 101-102) refuses implicit conversion to or from bool and char,
but no rule says whether `flag as u8`, `c as int` or `c as u8` compile or are E2020, though TYP-6 is not
marked as a complete list (TYP-5 is). Unit-only enums are settled by ENM-3 (p05 281-282). Searched p04, p05,
p06, p17a for `as` with bool or char: no example and no refusal. The finder noted it as not reported.

options:
[{'letter': 'A', 'summary': '`bool` converts with `as` to every integer type (false 0, true 1), and `char` to
every integer type as its scalar value, a narrower one keeping the low bits as every integer cast does. Into
`bool` or `char` stays as today (`x != 0`, `char.from_u32`).', 'code': "return (c as int) - ('0' as
int)\nreturn (visible as u8) | ((lit as u8) << 1)", 'outcome': 'Compiles; prints `7 1`.'}, {'letter': 'B',
'summary': 'The list is complete: only `char → u32` and `u8 → char`; everything else with a bool or char is
E2020.', 'code': "return ((c as u32) as int) - (('0' as u32) as int)\nreturn (1 if visible else 0) | ((1 if
lit else 0) << 1)", 'outcome': "The first program's casts are E2020, their help giving these lines; with them
it compiles and prints `7 1`."}]

recommendation:
{'letter': 'A', 'reason': 'B only forces a second cast through `u32` that loses the same bits, and a
conditional for what is a 0 or 1; A is one integer-cast rule for every whole-number value.', 'checklist': '2
(correct programs accepted), 5 (one cast rule), 6 (no exception without a reason)'}

