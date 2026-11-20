// The C++ side of the M12 FFI-41 test: two `const` objects of a namespace, one whose type has a
// `mutable` member.
#pragma once

struct Stats {
    int total;
    mutable int hits;
};

struct Plain {
    int total;
};

inline const Stats STATS = {3, 0};
inline const Plain PLAIN = {4};
