// The C++ side of the FFI-50 tests: it needs the project's define, include folder and flag (`static`,
// so each import's thunk file has its own, compiled with its own project's flags).
#pragma once
#include "detail.hpp"

static inline int scaled(int x) noexcept { return x * ENGINE_SCALE + detail_offset() + ENGINE_FLAG; }
