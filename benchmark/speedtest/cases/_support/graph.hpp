// The graph the C++ counterparts of the graph cases run on, laid out as `Graph` in cp-library's
// `CPLib.Graph` lays it out, so that the cases compare the two languages on one data structure:
// the edges are added in order, then placed per source vertex in one array, with vertex numbers
// and edge positions held in 4 bytes.
#include <cstdint>
#include <utility>
#include <vector>

namespace speedtest {

template <class C>
struct Graph {
    // Of length n + 1. The edges from vertex v are at positions [start[v], start[v + 1]).
    std::vector<int32_t> start;
    // The target vertex of each edge, in the order of their source vertices.
    std::vector<int32_t> to;
    // The data of each edge, in the order of `to`.
    std::vector<C> data;

    // Lay out the `n`-vertex graph whose i-th edge goes from `ends[i].first` to `ends[i].second`
    // and carries `edge_data[i]`, as `GraphBuilder::build` does.
    Graph(int32_t n, const std::vector<std::pair<int32_t, int32_t>>& ends, const std::vector<C>& edge_data)
        : start(n + 1, 0), to(ends.size()), data(ends.size()) {
        int32_t m = ends.size();
        for (auto [u, v] : ends) start[u + 1]++;
        for (int32_t v = 0; v < n; v++) start[v + 1] += start[v];
        // perm[p] is the edge placed at position p; next[v] the position the next edge from v takes.
        std::vector<int32_t> next(start.begin(), start.end() - 1), perm(m);
        for (int32_t i = 0; i < m; i++) perm[next[ends[i].first]++] = i;
        for (int32_t p = 0; p < m; p++) {
            to[p] = ends[perm[p]].second;
            data[p] = edge_data[perm[p]];
        }
    }
};

}  // namespace speedtest
