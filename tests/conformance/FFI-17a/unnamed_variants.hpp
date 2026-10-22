// The C++ side of reject_a_variant_the_overlay_does_not_name_whole.em: variants an overlay names
// wrongly, or whose alternative does not map by value.
#pragma once
#include <string>
#include <variant>

using Pair = std::variant<int, double>;
using Texty = std::variant<int, std::string>;
inline int first(Pair pair) noexcept { return static_cast<int>(pair.index()); }
inline int second(const Texty& texty) noexcept { return static_cast<int>(texty.index()); }
