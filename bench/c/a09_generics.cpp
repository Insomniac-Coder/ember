#include <cstdint>
#include <cstdio>
template <class T> static T larger(T a, T b) { return a > b ? a : b; }
int main() {
    int64_t best = 0, total = 0;
    for (int64_t i = 0; i < 200000000; i++) { best = larger<int64_t>(best, (i * 7919) % 100003); total += best; }
    std::printf("%lld %lld\n", (long long)best, (long long)total);
    return 0;
}
