# K34-which-types-take-using

Question: Which heap-owning types may name a memory manager with using (String, Deque, …)?
Severity: medium; impact rank 34
Members: J2-02, J5-09; primary J2-02
Why together: J5-09 (String) is J2-02's first case. Both edit TYP-46/ALC-1's closed list and E2114.
Critic's notes: Ask after K05. Use J5-09's parser example for String and J2-02's for the other collections. See the recommendation change: as worded, A is a path only the standard library can take.

## Finding J2-02
Rules: TYP-46, ALC-1, IV.1; file tasks/spec-0.9.9/parts/p04-types.md lines 11, 433-435; kind contradiction; severity medium
Quote from Hardened_11:
It may follow an `owns`, `shares`, `shares mut` or `thread_shared` type, a class type, or an `Array`, a `Map`
or a `Set`; after any other type it is `E2114`

The finder's question:
Does `name: String using LevelArena` compile, and do `Deque`, `SmallArray` and the other heap collections take
`using`?

Program:
```
struct Level:
    names: Array[String] using LevelArena
    title: String using LevelArena     # allowed or E2114?

fn load(ref mut current: LevelArena) -> Level:
    return Level(names=[] using current, title=String("Cave") using current)
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.75)
Reason:
TYP-46 (p04 430-435) and ALC-1 (p09 184-186) open with the property ('a type whose value has heap memory of
its own') and then give a closed list that leaves out String, Deque, SmallArray, BitSet and RingBuffer, with
'after any other type it is E2114'. ALC-1 itself (p09 205-206) says 'a value that grows (an Array, Map, Set or
String) made in it keeps the word', so a String is made in a manager there. IV.1's Collection row (p04 11)
lists String among the collections whose buffer comes 'from the memory manager its type names'. Verdicts 19,
24, 47 and 107 use Array and owns/shares only; none decides String or the other collections.

options:
[{'letter': 'A', 'summary': '`using` follows every type whose value itself holds heap memory: the owner words,
a class, and every standard collection, String, Deque, SmallArray, BitSet and RingBuffer included. The list
becomes examples of the property.', 'code': 'struct Level:\n    names: Array[String] using Arena\n    title:
String using Arena', 'outcome': "Compiles; prints `Cave 1`. `title`'s text lives in `level_mem`. `using` after
`int` or a struct stored inline stays E2114."}, {'letter': 'B', 'summary': 'Add String to the list (the rule
already makes one in a manager); the other heap collections stay E2114.', 'code': 'struct Level:\n    names:
Array[String] using Arena\n    title: String using Arena\n    queue: Deque[int] using Arena      # E2114',
'outcome': '`title` compiles as in A; `queue` is E2114, its buffer from the built-in manager only.'},
{'letter': 'C', 'summary': 'Keep the list as it is: String is E2114, and the sentence saying a String may be
made in a manager is removed.', 'code': 'struct Level:\n    names: Array[String] using Arena\n    title:
String                      # from the built-in manager', 'outcome': '`title: String using Arena` is E2114 ("a
`String` ... remove `using`"); the title\'s text always comes from the built-in manager.'}]

recommendation:
{'letter': 'A', 'reason': 'The rule already states the test (heap memory of its own); a closed list that
refuses a String, the most common heap value, while another sentence makes one in a manager is an exception
with no reason. A keeps one rule for every collection.', 'checklist': "5 (one meaning for every type), 6 (no
exception the rule's own wording does not need), 7 (general: every collection, not three), 13 (no type left on
a side path)"}

## Finding J5-09
Rules: ALC-1, TYP-46; file tasks/spec-0.9.9/parts/p09-memory.md lines 186, 208-209; kind contradiction; severity medium
Quote from Hardened_11:
`thread_shared` value, a class object, an `Array`, `Map` or `Set`) names the kind of memory manager
is freed together when the instance is cleared or ends; a value that grows (an `Array`, `Map`, `Set`
or `String`) made in it keeps the word, since its growth goes back to the instance; (2) a manager that

The finder's question:
Can a `String` take its memory from a manager? Does `text: String using LevelArena` compile, or is it E2114?

Program:
```
struct Token:
    kind: int
    text: String using LevelArena              # E2114 by the list, yet [ALC-1] speaks of a String made in a manager

fn first_token(ref mut arena: LevelArena, src: StrView) -> Token:
    return Token(kind=1, text=String(src) using arena)
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.7)
Reason:
The closed list in [TYP-46] (p04 433-435) and [ALC-1] leaves out `String` and makes anything not on it E2114.
Yet [ALC-1] (1) (p09 208-209) describes a `String` made in a manager, the E2114 row (p17a 83) describes
`using` "after a type with no heap memory of its own", and [TXT-1] says a `String` owns a heap buffer. No
verdict names `String`: 19, 24, 47 and 107 name `Array`, lists and values that grow. The two sides decide
`text: String using Arena` opposite ways, so it is the owner's.

options:
[{'letter': 'A', 'summary': '`String` joins the types `using` may follow, beside `Array`, `Map` and `Set`.',
'code': 'struct Token:\n    kind: int\n    text: String using Arena\n\nfn read_token(ref mut arena: Arena,
src: StrView) -> Token:\n    return Token(kind=1, text=String(src) using arena)\n\nfn main():\n    arena =
Arena.with_capacity(64 * 1024)\n    t = read_token(ref mut arena, "let")\n    println(t.text)', 'outcome':
"Compiles and prints let. The text's buffer comes from `arena`, and its growth goes back to it. The `Token`
lives no longer than `arena` (E3063 otherwise), as every value holding an arena-made value does."}, {'letter':
'B', 'summary': '`String` keeps the built-in manager, and the arena sentence drops `String`.', 'code': 'struct
Token:\n    kind: int\n    text: String                         # from the built-in manager\n\nfn
read_token(src: StrView) -> Token:\n    return Token(kind=1, text=String(src))\n\nfn main():\n    t =
read_token("let")\n    println(t.text)', 'outcome': 'In the example, `text: String using Arena` is E2114: "a
`String` takes its memory from the built-in manager; remove `using`". This version prints let, with the text
outside the arena.'}]

recommendation:
{'letter': 'A', 'reason': 'A `String` is a growable buffer exactly as an `Array` is, and the specification
already describes a `String` made in an arena. One rule covers every growable buffer, and a parser can keep
its tokens and their text in one arena.', 'checklist': '5 (one word, one meaning for every type), 6 (the
nearest rule: `Array`, `Map` and `Set`), 7'}

