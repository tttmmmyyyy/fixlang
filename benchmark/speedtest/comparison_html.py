"""Render the latest row of the benchmark log as a page comparing Fix with C, Rust and C++.

Reads `log.csv` (one row per measured commit, one column per case and metric) and writes a
self-contained HTML page. For each reference language, the page draws one chart and one table:
a row per case that carries a counterpart in that language, holding how many cycles and how
many instructions the Fix program took against its counterpart.

    python3 comparison_html.py [--log log.csv] [--out comparison.html]
"""

import argparse
import csv
import html
import math
import re
import tempfile
from pathlib import Path

REPOSITORY = "https://github.com/tttmmmyyyy/fixlang"

# The metrics the page compares, as the suffix of their columns in `log.csv` and the name the
# page gives them.
METRICS = [("cycles", "Cycles"), ("inst", "Instructions")]


class Comparison:
    """One reference language the page compares Fix with: which columns of the log hold each
    side, and what the page says about how the two sides were built."""

    def __init__(self, language, title, fix_suffix, case_pattern, conditions):
        # The suffix of the reference's columns in `log.csv`, after the metric.
        self.language = language
        self.title = title
        # The suffix of the Fix program's columns: empty for the case as the harness builds it,
        # `-unchecked` for the case built with `--no-runtime-check`.
        self.fix_suffix = fix_suffix
        # The cases this comparison draws, among those that carry a counterpart in the language.
        self.case_pattern = re.compile(case_pattern)
        # HTML paragraphs saying how both sides were built and what the comparison stands for.
        self.conditions = conditions


COMPARISONS = [
    Comparison(
        "c", "Fix vs C", "-unchecked", r"^(?!cp_)",
        "<p>Small programs written the same way in both languages: the classic cross-language "
        "benchmarks (n-body, mandelbrot, fannkuch-redux, binary-trees, recursive Fibonacci) and a "
        "few loops over arrays.</p>"
        "<p>C does not check array bounds, so the Fix programs are built with "
        "<code>--no-runtime-check</code>, which removes the bounds checks and the other checks "
        "that abort a program. The C programs are built with <code>gcc -O3 -march=native</code>."
        "</p>"),
    Comparison(
        "rust", "Fix vs Rust", "", r"^(?!cp_)",
        "<p>The same programs as the comparison with C, written in safe Rust.</p>"
        "<p>Rust checks array bounds, so the Fix programs are built with their runtime checks. "
        "The Rust programs are built with <code>rustc -O -C target-cpu=native</code>.</p>"),
    Comparison(
        "cpp", "cp-library vs AtCoder Library (C++)", "-unchecked", r"^cp_",
        "<p><a href=\"https://github.com/tttmmmyyyy/cp-library\">cp-library</a>, the Fix library "
        "for competitive programming, against <a href=\"https://github.com/atcoder/ac-library\">"
        "AtCoder Library</a>, the C++ library it follows. Both sides run the same algorithm on "
        "the same large random input, and only the part that calls the library is counted: "
        "generating the input is left out on both sides.</p>"
        "<p>The Fix programs are built with <code>--no-runtime-check</code>, since C++ does not "
        "check array bounds. The C++ programs are built with <code>g++ -O3 -march=native "
        "-std=gnu++20</code>.</p>"
        "<p>The Fix compiler installed on AtCoder's judge is version 1.2.0, and the compiler has "
        "been optimized substantially since then, so these numbers are not reached on the judge. "
        "cp-library itself requires Fix 1.6.0 or later.</p>"),
]


def read_log(path):
    with path.open(encoding="utf-8") as f:
        rows = [r for r in csv.reader(f) if any(c.strip() for c in r)]
    if len(rows) < 2:
        raise SystemExit(f"{path} holds no measured row")
    return rows[0], rows[1:]


def latest_row(header, body):
    """The last row measured from a committed tree, as a map from column name to cell.

    A row from a dirty tree measured a compiler no commit holds, so the page names no commit for
    it, and it is passed over.
    """
    for row in reversed(body):
        if not row[0].strip().endswith("(dirty)"):
            return {name.strip(): (row[i].strip() if i < len(row) else "")
                    for i, name in enumerate(header)}
    raise SystemExit("the log holds no row measured from a committed tree")


def number(cell):
    return int(cell) if cell.isdigit() else None


def build_sections(row):
    """For each comparison, the cases it draws, each with the Fix count and the reference count
    of every metric. A count the row left empty is `None`."""
    sections = []
    for comparison in COMPARISONS:
        cases = set()
        for metric, _ in METRICS:
            tail = f"-{metric}-{comparison.language}"
            cases |= {name[: -len(tail)] for name in row if name.endswith(tail) and row[name]}
        cases = sorted(c for c in cases if comparison.case_pattern.search(c))
        rows = []
        for case in cases:
            counts = {}
            for metric, _ in METRICS:
                fix = number(row.get(f"{case}-{metric}{comparison.fix_suffix}", ""))
                reference = number(row.get(f"{case}-{metric}-{comparison.language}", ""))
                counts[metric] = (fix, reference)
            rows.append((case, counts))
        if rows:
            sections.append((comparison, rows))
    return sections


def ratio(counts):
    fix, reference = counts
    if not fix or not reference:
        return None
    return fix / reference


def geometric_mean(values):
    values = [v for v in values if v is not None]
    if not values:
        return None
    return math.exp(sum(math.log(v) for v in values) / len(values))


def format_ratio(value):
    return "&ndash;" if value is None else f"{value:.2f}&times;"


def format_count(value):
    return "&ndash;" if value is None else f"{value:,}"


def display_name(case):
    return case[len("cp_"):] if case.startswith("cp_") else case


# The geometry of a chart, in SVG user units. The SVG scales to the width of the page.
LABEL_WIDTH = 160
PLOT_WIDTH = 520
CHART_WIDTH = LABEL_WIDTH + PLOT_WIDTH + 20
BAR_HEIGHT = 9
ROW_HEIGHT = 2 * BAR_HEIGHT + 10
AXIS_HEIGHT = 34


def chart(rows):
    """An SVG drawing a pair of bars per case: the log of the ratio of each metric, Fix over the
    reference, with 1 at the centre. A bar to the left is a case where Fix took less."""
    logs = [abs(math.log2(r)) for _, counts in rows for metric, _ in METRICS
            if (r := ratio(counts[metric])) is not None]
    # The axis reaches the next power of two past the farthest bar, and at least to 2x either
    # way, so that a chart of near-equal cases does not magnify the noise.
    reach = max(1, math.ceil(max(logs, default=0) + 1e-9))
    scale = (PLOT_WIDTH / 2) / reach
    centre = LABEL_WIDTH + PLOT_WIDTH / 2
    height = AXIS_HEIGHT + ROW_HEIGHT * len(rows) + 6
    out = [f'<svg class="chart" viewBox="0 0 {CHART_WIDTH} {height}" role="img" '
           f'aria-label="Ratio of Fix to the reference, per case">']
    for k in range(-reach, reach + 1):
        x = centre + k * scale
        label = f"{2 ** k:g}&times;" if k >= 0 else f"1/{2 ** -k}"
        out.append(f'<line class="{"one" if k == 0 else "grid"}" x1="{x:.1f}" y1="{AXIS_HEIGHT - 8}"'
                   f' x2="{x:.1f}" y2="{height}"/>')
        out.append(f'<text class="tick" x="{x:.1f}" y="{AXIS_HEIGHT - 14}">{label}</text>')
    out.append(f'<text class="side" x="{LABEL_WIDTH + 4}" y="12">&larr; Fix faster</text>')
    out.append(f'<text class="side end" x="{LABEL_WIDTH + PLOT_WIDTH - 4}" y="12">'
               f'Fix slower &rarr;</text>')
    for i, (case, counts) in enumerate(rows):
        top = AXIS_HEIGHT + i * ROW_HEIGHT + 4
        out.append(f'<text class="case" x="{LABEL_WIDTH - 8}" y="{top + BAR_HEIGHT + 4}">'
                   f'{html.escape(display_name(case))}</text>')
        for j, (metric, name) in enumerate(METRICS):
            r = ratio(counts[metric])
            if r is None:
                continue
            width = math.log2(r) * scale
            x = centre + min(0, width)
            y = top + j * BAR_HEIGHT
            out.append(f'<rect class="bar {metric}" x="{x:.1f}" y="{y}" '
                       f'width="{max(abs(width), 1):.1f}" height="{BAR_HEIGHT - 1}">'
                       f'<title>{html.escape(display_name(case))}: {name.lower()} '
                       f'{r:.2f}&times;</title></rect>')
    out.append("</svg>")
    return "".join(out)


def table(comparison, rows):
    reference = comparison.title.split(" vs ")[1]
    head = "".join(f'<th colspan="3" class="group {metric}">{name}</th>' for metric, name in METRICS)
    sub = "".join(f"<th>Fix</th><th>{html.escape(reference)}</th><th>Ratio</th>"
                  for _ in METRICS)
    body = []
    for case, counts in rows:
        cells = "".join(
            f"<td>{format_count(counts[m][0])}</td><td>{format_count(counts[m][1])}</td>"
            f"<td class=\"ratio\">{format_ratio(ratio(counts[m]))}</td>" for m, _ in METRICS)
        body.append(f"<tr><th scope=\"row\">{html.escape(display_name(case))}</th>{cells}</tr>")
    means = "".join(f'<td colspan="2"></td><td class="ratio">'
                    f'{format_ratio(geometric_mean(ratio(c[m]) for _, c in rows))}</td>'
                    for m, _ in METRICS)
    return (f'<div class="table-wrap"><table><thead><tr><th rowspan="2">Case</th>{head}</tr>'
            f'<tr>{sub}</tr></thead><tbody>{"".join(body)}</tbody>'
            f'<tfoot><tr><th scope="row">Geometric mean</th>{means}</tr></tfoot></table></div>')


def section(comparison, rows):
    mean = geometric_mean(ratio(c["cycles"]) for _, c in rows)
    summary = ("" if mean is None else
               f'<p class="summary">Geometric mean of the cycle ratios: '
               f'<strong>{format_ratio(mean)}</strong> over {len(rows)} programs.</p>')
    return (f'<section><h2>{html.escape(comparison.title)}</h2>{comparison.conditions}'
            f'{summary}{chart(rows)}{table(comparison, rows)}</section>')


def render(row):
    commit = row["commit"]
    cpu = row.get("cpu", "")
    sections = "".join(section(c, rows) for c, rows in build_sections(row))
    legend = "".join(f'<span class="key"><span class="swatch {metric}"></span>{name}</span>'
                     for metric, name in METRICS)
    return TEMPLATE.format(
        commit_link=f'<a href="{REPOSITORY}/commit/{commit}"><code>{commit[:9]}</code></a>',
        cpu=html.escape(cpu) if cpu else "an unrecorded processor",
        legend=legend, sections=sections)


TEMPLATE = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Fix benchmarks</title>
<style>
:root {{
  --bg: #ffffff; --fg: #1f2328; --muted: #59636e; --rule: #d1d9e0; --grid: #eaeef2;
  --cycles: #0969da; --inst: #bf8700; --stripe: #f6f8fa;
}}
@media (prefers-color-scheme: dark) {{
  :root:not([data-theme="light"]) {{
    --bg: #0d1117; --fg: #e6edf3; --muted: #9198a1; --rule: #3d444d; --grid: #21262d;
    --cycles: #4493f8; --inst: #d29922; --stripe: #151b23;
  }}
}}
:root[data-theme="dark"] {{
  --bg: #0d1117; --fg: #e6edf3; --muted: #9198a1; --rule: #3d444d; --grid: #21262d;
  --cycles: #4493f8; --inst: #d29922; --stripe: #151b23;
}}
* {{ box-sizing: border-box; }}
body {{ margin: 0; background: var(--bg); color: var(--fg);
  font: 16px/1.55 -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif; }}
main {{ max-width: 900px; margin: 0 auto; padding: 32px 16px 64px; }}
h1 {{ font-size: 1.9rem; margin: 0 0 8px; }}
h2 {{ font-size: 1.35rem; margin: 48px 0 8px; padding-bottom: 6px; border-bottom: 1px solid var(--rule); }}
p {{ margin: 8px 0; }}
a {{ color: var(--cycles); }}
code {{ font: 0.88em ui-monospace, SFMono-Regular, Menlo, monospace; }}
.lead {{ color: var(--muted); }}
.summary {{ margin-top: 16px; }}
.legend {{ display: flex; gap: 20px; margin: 16px 0 0; font-size: 0.92rem; }}
.key {{ display: inline-flex; align-items: center; gap: 6px; }}
.swatch {{ width: 14px; height: 10px; display: inline-block; }}
.swatch.cycles, .bar.cycles {{ background: var(--cycles); fill: var(--cycles); }}
.swatch.inst, .bar.inst {{ background: var(--inst); fill: var(--inst); }}
.chart {{ width: 100%; height: auto; margin: 12px 0 8px; display: block; }}
.chart text {{ fill: var(--fg); font-size: 12px; }}
.chart .tick {{ text-anchor: middle; fill: var(--muted); }}
.chart .side {{ fill: var(--muted); }}
.chart .side.end {{ text-anchor: end; }}
.chart .case {{ text-anchor: end; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }}
.chart .grid {{ stroke: var(--grid); }}
.chart .one {{ stroke: var(--muted); }}
.table-wrap {{ overflow-x: auto; margin-top: 8px; }}
table {{ border-collapse: collapse; font-size: 0.85rem; font-variant-numeric: tabular-nums; width: 100%; }}
th, td {{ padding: 4px 8px; text-align: right; white-space: nowrap; border-bottom: 1px solid var(--grid); }}
thead th {{ border-bottom: 1px solid var(--rule); }}
th[scope="row"], thead th:first-child {{ text-align: left; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-weight: normal; }}
th.group.cycles {{ color: var(--cycles); }}
th.group.inst {{ color: var(--inst); }}
tbody tr:nth-child(even) {{ background: var(--stripe); }}
td.ratio {{ font-weight: 600; }}
tfoot th, tfoot td {{ border-top: 1px solid var(--rule); border-bottom: none; font-weight: 600; }}
.method {{ color: var(--muted); font-size: 0.92rem; }}
</style>
</head>
<body>
<main>
<h1>Fix benchmarks</h1>
<p class="lead">How long programs compiled by the Fix compiler take, against the same programs
written in C, Rust and C++. Measured at commit {commit_link} on {cpu}.</p>
<p>Each chart shows, per program, the ratio of the Fix program's count to the other language's,
on a logarithmic axis: a bar to the left of 1&times; is a program where Fix took less, and twice
as fast and twice as slow are bars of the same length.</p>
<div class="legend">{legend}</div>
{sections}
<h2>How the numbers are taken</h2>
<div class="method">
<p>Every count is read from the hardware performance counters with <code>perf stat</code>, in
user mode, on one core with nothing else running on it. Cycles are what the program took; the
lowest count of five windows of runs is kept, since other work only ever adds to it.
Instructions are the instructions the program retired, a count the rest of the machine does not
move.</p>
<p>The two can disagree: a program that waits on memory, or that mispredicts branches, takes
more cycles per instruction than one that does not. Where code lands in memory can also move a
cycle count by a few tens of percent without changing a single instruction.</p>
<p>The Fix programs are built with <code>fix build -O experimental</code>. The programs and the
harness are in <a href="{repository}/tree/main/benchmark/speedtest">benchmark/speedtest</a>;
each Fix program sits beside its counterparts, as <code>main.fix</code> and
<code>ref.c</code>, <code>ref.rs</code> or <code>ref.cpp</code>.</p>
</div>
</main>
</body>
</html>
""".replace("{repository}", REPOSITORY)


def self_check():
    """Render a fixture log, so that a page bug shows on every invocation rather than on the
    published page."""
    with tempfile.TemporaryDirectory() as tmp:
        log = Path(tmp) / "log.csv"
        log.write_text(
            "commit,cpu,fib-inst,fib-cycles,fib-inst-unchecked,fib-cycles-unchecked,"
            "fib-inst-c,fib-cycles-c,fib-inst-rust,fib-cycles-rust,cp_uf-inst-unchecked,"
            "cp_uf-cycles-unchecked,cp_uf-inst-cpp,cp_uf-cycles-cpp\n"
            "1111111111111111111111111111111111111111,Zen,100,200,90,180,90,90,50,,"
            "300,400,150,100\n"
            "2222222222222222222222222222222222222222(dirty),Zen,1,1,1,1,1,1,1,1,1,1,1,1\n",
            encoding="utf-8")
        header, body = read_log(log)
    row = latest_row(header, body)
    # A dirty row names no commit, so the page is drawn from the row before it.
    assert row["commit"].startswith("1111"), row["commit"]
    sections = {c.language: rows for c, rows in build_sections(row)}
    # Fix is read from the unchecked build against C and C++, and from the checked build
    # against Rust.
    assert sections["c"] == [("fib", {"cycles": (180, 90), "inst": (90, 90)})], sections["c"]
    # A cycle count the run withheld leaves no ratio, and the case stays in the table.
    assert sections["rust"] == [("fib", {"cycles": (200, None), "inst": (100, 50)})], sections
    assert sections["cpp"] == [("cp_uf", {"cycles": (400, 100), "inst": (300, 150)})], sections
    assert ratio(sections["rust"][0][1]["cycles"]) is None
    assert abs(geometric_mean([2.0, 0.5]) - 1.0) < 1e-12
    page = render(row)
    assert "4&times;" in page and "uf" in page, "the axis reaches 4x for a ratio of 4"


def main():
    self_check()
    here = Path(__file__).resolve().parent
    ap = argparse.ArgumentParser()
    ap.add_argument("--log", default=here / "log.csv", type=Path)
    ap.add_argument("--out", default=here / "comparison.html", type=Path)
    args = ap.parse_args()
    header, body = read_log(args.log)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(render(latest_row(header, body)), encoding="utf-8")


if __name__ == "__main__":
    main()
