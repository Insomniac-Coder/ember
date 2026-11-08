# K48-migrate-from-ownership

Question: How does migrate_from keep an open socket, and what happens to old fields?
Severity: medium; impact rank 48
Members: J7-15; primary J7-15
Why together: Single question.
Critic's notes: Option A's code writes `owns` before constructor arguments, which verdict 159 refused (verdict clash). Write `Session(sock=old.sock, target=old.target, hp=Health(value=old.hp))` before showing it.

## Finding J7-15
Rules: HR-16, HR-2a, HR-14, HR-15; file tasks/spec-0.9.9/parts/p20-annex-b.md lines 168-170, 31-33; kind undecided_case; severity medium
Quote from Hardened_11:
`static fn migrate_from(old: OldSelf) -> Result[Self, ReloadError]` replaces field-wise / migration for its
type. `OldSelf` is the previous schema as a generated struct `Old<Name>`, visible / only in that function;
`old` is a read-only view of the old instance.

The finder's question:
A type with `migrate_from` that also holds an owned resource (`owns Socket`, a `File`, a `shares` owner): how
does `migrate_from` carry the resource into the new instance from a read-only view, and in reclaim, are the
old instance's fields ended (their `drop` run, counts decremented) or freed without drop?

Program:
```
class Session:
    sock: owns Socket
    target: shares Player
    hp: Health              # was `hp: int`

    static fn migrate_from(old: OldSession) -> Result[Session, ReloadError]:
        return Ok(Session(sock=???, target=shares old.target, hp=Health(old.hp)))
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.6)
Reason:
HR-16 (p20 168-170) gives `migrate_from` 'a read-only view of the old instance', so an owned field that is not
`Clone` (an open socket) can be neither handed over nor copied, though the annex promises to keep 'open
resources' (p20 5-6). HR-2a (p20 31-33) has reclaim free the old version 'field by field without the type's
own drop' and end only 'removed fields', and for a `migrate_from` type, which 'replaces field-wise migration',
it does not say which old fields count as removed. If they are not ended, `shares old.target` leaves one count
too many and the old socket is never closed. If they are, nothing could have been carried over. Searched HR-2,
HR-2a, the HR-14 table, HR-15, HR-16 and HR-34 to HR-43.

options:
[{'letter': 'A', 'summary': '`migrate_from` takes the old instance with `owns`; what it leaves is ended at
reclaim', 'code': "    static fn migrate_from(owns old: OldSession) -> Result[Session, ReloadError]:\n
return Ok(Session(sock=owns old.sock, target=owns old.target, hp=Health(value=old.hp)))\n# what `old` still
owns after the call is ended in reclaim (drops run, counts lowered),\n# as an `owns` parameter's leftovers
are; if prepare fails, nothing of `old` is ended or taken", 'outcome': 'Compiles. After the reload the session
keeps socket 7 open, with the same player owner and no extra count, and nothing is printed. A field not taken
(say `target`, if the function made `shares old.target` instead) is ended at reclaim, its count lowered.'},
{'letter': 'B', 'summary': 'Keep the read-only view; such a type cannot declare migrate_from', 'code': '
static fn migrate_from(old: OldSession) -> Result[Session, ReloadError]:\n        ...\n# refused when
compiling: "`Session` owns `sock`, a `Socket` that is not `Clone`, which a\n#  read-only `old` cannot hand on;
restart to change `hp`"; the old instance\'s fields are all\n#  ended at reclaim', 'outcome': 'The type with
the socket does not compile with `migrate_from`, so the edit needs a restart, which closes every connection. A
type with only copyable fields migrates, and its old fields are ended at reclaim.'}]

recommendation:
{'letter': 'A', 'reason': "It reuses the word the language already has for 'this function takes the value
over', and keeps the annex's promise to keep open resources. Reclaim then ends exactly what nobody took, as
any `owns` parameter's leftovers are.", 'checklist': '2 (the correct migration compiles), 5 (`owns` means the
same here as everywhere), 11 (the existing spelling of handing over), 13 (no special path: leftovers ended as
for any `owns` parameter)'}

