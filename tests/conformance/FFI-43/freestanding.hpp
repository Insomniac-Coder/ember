// The C++ side of the FFI-43 E5061 test: a computed exception specification.
#pragma once
inline int add(int a, int b) noexcept(sizeof(int) == 4) { return a + b; }
inline int sub(int a, int b) noexcept(sizeof(int) == 4) { return a - b; }
