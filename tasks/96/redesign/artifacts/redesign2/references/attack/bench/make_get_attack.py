# Writes get_attack.c: ../../bench/get.c plus -DSLOTVAR (a load-time slot number for Pool[Pos]). Run: python -I make_get_attack.py
src = open("../../bench/get.c").read()
reps = [
    ("/* get.c: A5 / ODR-286.",
     "/* get_attack.c (written by make_get_attack.py): ../../bench/get.c plus -DSLOTVAR, used with -DTYPED: the pool\n"
     "   table's slot number for Pool[Pos] is not a compile-time constant but a number assigned when the program is\n"
     "   loaded (separate packages, hot reload), so each get loads it and checks it against the table's length.\n"
     "   get.c: A5 / ODR-286."),
    ("GETATTR Pos *get_pos(uint32_t e) {\n    Pool *p = pools[T_POS];",
     "#ifdef SLOTVAR\nuint32_t pos_slot = T_POS;    /* not static, not const: assigned at load time */\nuint32_t n_slots = NTYPES;\n#endif\n"
     "GETATTR Pos *get_pos(uint32_t e) {\n#ifdef SLOTVAR\n    if (pos_slot >= n_slots) panic_(\"no Pos slot\");\n    Pool *p = pools[pos_slot];\n#else\n    Pool *p = pools[T_POS];\n#endif"),
    ("#elif defined(TYPED)\n    const char *name = \"get-typed\";",
     "#elif defined(TYPED) && defined(SLOTVAR)\n    const char *name = \"get-typed-slotvar\";\n#elif defined(TYPED)\n    const char *name = \"get-typed\";"),
]
for a, b in reps:
    assert src.count(a) == 1, a
    src = src.replace(a, b)
open("get_attack.c", "w").write(src)
print("wrote get_attack.c")
