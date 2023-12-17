// C++ implementation of ./examples/prime_loop.fix.
#include <stdio.h>
#include <vector>
#include <cinttypes>

using namespace std;

using vector_bool = vector<uint8_t>; // Avoid pack representation.

vector_bool is_prime(int64_t n) {
    vector_bool arr(n, 1);
    arr[0] = 0;
    arr[1] = 0;
    for (int64_t i = 2; i*i <= n; i++) {
        if (arr[i]) {
            for (int64_t q = i+i; q < n; q += i) {
                arr[q] = 0;
            }
        }
    }
    return arr;
}

template<typename T>
int64_t count(T elem, const vector<T>& arr) {
    int64_t sum = 0;
    for (int64_t i = 0; i < arr.size(); i++) {
        if (arr[i] == elem) {
            sum++;
        }
    }
    return sum;
}

int main(void) {
    vector_bool table = is_prime(50000000);
    int64_t prime_count = count((uint8_t)1, table);
    printf("%" PRId64 "\n", prime_count);
    return 0;
}