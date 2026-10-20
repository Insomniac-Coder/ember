// The C++ side of reject_a_parameter_that_maps_to_nothing.em.
#pragma once

struct Text {
    int size() const noexcept { return 4; }
};

inline int length(int (Text::*text)() const) { return (Text().*text)(); }
inline int twice(int value) noexcept { return value * 2; }
