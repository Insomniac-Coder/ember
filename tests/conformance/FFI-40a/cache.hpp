// The C++ side of the FFI-40a tests: a class with `mutable` state, whose `const` methods change
// it in each of the ways a body can, or change nothing.
#pragma once

struct Tally {
    int hits = 0;
    void bump() noexcept { ++hits; }
    int seen() const noexcept { return hits; }
};

inline void touch(int& value) noexcept { ++value; }
inline int peek(const int& value) noexcept { return value; }

class Cache {
public:
    Cache() noexcept {}
    ~Cache() {}
    // Each changes its mutable state: `++`, `=`, `+=`, a non-`const` method of a field, a field
    // handed on by reference, and a call of a method of its own that does.
    int lookup(int key) const noexcept {
        ++lookups_;
        return key * 2;
    }
    void reset() const noexcept { lookups_ = 0; }
    void add(int count) const noexcept { lookups_ += count; }
    void note() const noexcept { tally_.bump(); }
    void hand_on() const noexcept { touch(lookups_); }
    int twice(int key) const noexcept { return lookup(key) + lookup(key); }
    // Each changes nothing: a read, a `const` method of a field, a field handed on by `const`
    // reference, writes of a local only, and calls of methods of its own that change nothing.
    int lookups() const noexcept { return lookups_; }
    int hits() const noexcept { return tally_.seen(); }
    int peeked() const noexcept { return peek(lookups_); }
    int scaled(int by) const noexcept {
        int local = lookups_;
        local *= by;
        ++local;
        return local;
    }
    int total() const noexcept { return lookups() + hits(); }

private:
    mutable int lookups_ = 0;
    mutable Tally tally_;
};

// A `mutable` member of a base is the derived class's state too.
struct Seen {
    mutable int seen = 0;
};

struct Watcher : Seen {
    Watcher() noexcept {}
    int look() const noexcept { return ++seen; }
    int count() const noexcept { return seen; }
};
