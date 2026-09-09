# `[MOD-2]` with `[STR-1]` — one struct carrying every visibility class a
# field can have, so the matrix can be tested against a single declaration.
pub struct Panel:
    pub w: i32                 # public: read and write anywhere
    pub(package) h: i32        # package: this build is one package
    pub(read) title: i32       # readable anywhere, writable only in here
    secret: i32                # private to this module

pub fn make(w: i32, h: i32, title: i32, secret: i32) -> Panel:
    return Panel(w, h, title, secret)

pub fn secret_of(p: Panel) -> i32:
    # In the declaring module the private field reads normally.
    return p.secret

pub fn retitle(p: Panel, t: i32) -> Panel:
    # And `pub(read)` is writable in here, which is the whole point of it.
    q = p
    q.title = t
    return q
