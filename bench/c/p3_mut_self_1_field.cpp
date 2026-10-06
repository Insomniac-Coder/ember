#include <cstdint>
#include <cstdio>
#include <vector>

static int64_t step(int64_t x) { return x + 1; }

struct Thing {
    std::vector<int64_t> a;
    int64_t count = 0;
    void bump() { count += 1; }
};

int main() {
    std::vector<Thing*> things;
    for (int k = 0; k < 2; k++) things.push_back(new Thing());
    for (int64_t i = 0; i < 100000000; i++) {
        Thing* t = things[i % 2];
        t->bump();
    }
    std::printf("%lld\n", (long long)(things[0]->count + things[1]->count));
    return 0;
}
