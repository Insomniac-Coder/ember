// The C++ side of reject_a_parameter_that_maps_to_nothing.em.
#pragma once
#include <string>

inline int length(std::string text) { return static_cast<int>(text.size()); }
inline int twice(int value) noexcept { return value * 2; }
