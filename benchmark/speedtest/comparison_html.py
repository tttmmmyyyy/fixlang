"""Render the latest row of the benchmark log as a page comparing Fix with C, Rust and C++.

Reads `log.csv` (one row per measured commit, one column per case and metric) and writes a
self-contained HTML page. The page opens on one chart holding every pair of a case and a language
it has a counterpart in, sorted by how much faster the Fix program is, and explains the
comparisons and the measurement below it.

    python3 comparison_html.py [--log log.csv] [--out comparison.html]
"""

import argparse
import csv
import html
import math
import tempfile
from pathlib import Path

REPOSITORY = "https://github.com/tttmmmyyyy/fixlang"

# The metrics the page compares, as the suffix of their columns in `log.csv` and the name the
# page gives them. The chart is sorted by the last one.
METRICS = [("cycles", "Cycles"), ("inst", "Instructions")]

# What hovering over a metric in the legend says about it.
METRIC_NOTES = {
    "cycles": "How long the program ran, counted in CPU cycles so that the clock speed does not "
              "matter. The lowest of five measurements.",
    "inst": "The number of instructions the program executed. Memory stalls, branch "
            "mispredictions and code placement do not affect it, so it is steadier than the "
            "cycle count.",
}
SORT_METRIC = "inst"


class Comparison:
    """One reference language the page compares Fix with: which columns of the log hold each
    side, and what the page says about how the two sides were built."""

    def __init__(self, key, language, title, label, compiler, fix_suffix, fix_build, draws,
                 summary, conditions):
        # The name of the comparison on the page, which its colour is keyed on.
        self.key = key
        # The suffix of the reference's columns in `log.csv`, after the metric.
        self.language = language
        self.title = title
        # The name of the reference on a row of the chart.
        self.label = label
        # The compiler that builds the reference: its name, which `summary` and `conditions`
        # write as `{compiler}`, followed by its version where the log has one.
        self.compiler = compiler
        # The suffix of the Fix program's columns: empty for the case as the harness builds it,
        # `-unchecked` for the case built with `--no-runtime-check`.
        self.fix_suffix = fix_suffix
        # How the Fix side was built, in a few words for the tooltip of a row.
        self.fix_build = fix_build
        # Whether this comparison draws a case that carries a counterpart in the language, given
        # the case's name and its directory.
        self.draws = draws
        # HTML paragraphs saying how both sides were built and what the comparison stands for.
        self.conditions = conditions
        # What hovering over the comparison in the legend says about it: `conditions` in brief.
        self.summary = summary


# What the tooltip of a case's rows adds about the case, where the comparison's own description
# leaves out something a reader would assume otherwise.
CASE_NOTES = {
    "cp_bipartite": "AtCoder Library has no bipartite matching, so the C++ side solves it\n"
                    "as a maximum flow with mf_graph, as cp-library does.",
}


def calls_acl(case_dir):
    """Whether the C++ counterpart in `case_dir` calls AtCoder Library, rather than the C++
    standard library or code of its own."""
    reference = case_dir / "ref.cpp"
    return reference.exists() and "#include <atcoder/" in reference.read_text(encoding="utf-8")


COMPARISONS = [
    Comparison(
        "c", "c", "Fix vs C", "C", "gcc", "-unchecked", "Fix built with --no-runtime-check",
        lambda case, _: not case.startswith("cp_"),
        "Programs from the Computer Language Benchmarks Game, recursive Fibonacci, and a few "
        "loops over arrays. C does not check array bounds, so Fix is built with "
        "--no-runtime-check. C is built with {compiler} -O3 -march=native.",
        "<p>Small programs written the same way in both languages: programs from the "
        "<a href=\"https://benchmarksgame-team.pages.debian.net/benchmarksgame/\">Computer "
        "Language Benchmarks Game</a> (n-body, mandelbrot, fannkuch-redux, binary-trees), "
        "recursive Fibonacci, and a few loops over arrays.</p>"
        "<p>C does not check array bounds, so the Fix programs are built with "
        "<code>--no-runtime-check</code>. This option turns off bounds checks and the other "
        "checks that stop a program on an error. The C programs are built with "
        "{compiler} <code>-O3 -march=native</code>.</p>"),
    Comparison(
        "rust", "rust", "Fix vs Rust", "Rust", "rustc", "", "Fix built with runtime checks",
        lambda case, _: not case.startswith("cp_"),
        "The same programs, written in safe Rust. Rust checks array bounds, so Fix keeps its "
        "runtime checks. Rust is built with {compiler} -O -C target-cpu=native.",
        "<p>The same programs as above, written in safe Rust.</p>"
        "<p>Rust checks array bounds, so the Fix programs keep their runtime checks. The Rust "
        "programs are built with {compiler} <code>-O -C target-cpu=native</code>.</p>"),
    Comparison(
        "acl", "cpp", "cp-library vs AtCoder Library (C++)", "ACL", "g++", "-unchecked",
        "Fix built with --no-runtime-check",
        lambda case, case_dir: case.startswith("cp_") and calls_acl(case_dir),
        "cp-library, a Fix library for competitive programming, against AtCoder Library on "
        "large random inputs. Only the library calls are measured. C++ does not check array "
        "bounds, so Fix is built with --no-runtime-check. C++ is built with {compiler} -O3 "
        "-march=native.",
        "<p><a href=\"https://github.com/tttmmmyyyy/cp-library\">cp-library</a>, a Fix library "
        "for competitive programming, against <a href=\"https://github.com/atcoder/ac-library\">"
        "AtCoder Library</a>, the C++ library that AtCoder provides for its contests. Both run "
        "the same algorithm on the same large random input. Only the library calls are "
        "measured; generating the input is not.</p>"
        "<p>C++ does not check array bounds, so the Fix programs are built with "
        "<code>--no-runtime-check</code>. The C++ programs are built with {compiler} <code>-O3 "
        "-march=native -std=gnu++20</code>.</p>"
        "<p>AtCoder's judge runs Fix 1.2.0, and the compiler has become much faster since then, "
        "so these numbers are not reached there.</p>"),
    Comparison(
        "cpp", "cpp", "cp-library vs C++", "C++", "g++", "-unchecked",
        "Fix built with --no-runtime-check",
        lambda case, case_dir: case.startswith("cp_") and not calls_acl(case_dir),
        "cp-library against C++ for the functions AtCoder Library does not have: the C++ "
        "standard library, or a plain implementation. Only the function calls are measured. "
        "Fix is built with --no-runtime-check; C++ with {compiler} -O3 -march=native.",
        "<p>cp-library against C++ for the functions AtCoder Library does not have. The C++ side "
        "is the C++ standard library (<code>std::next_permutation</code>) or a plain "
        "implementation of the same algorithm (Dijkstra's algorithm on "
        "<code>std::priority_queue</code>, Bellman-Ford). As with AtCoder Library, both run on "
        "the same large random input, and only the function calls are measured.</p>"
        "<p>The Fix programs are built with <code>--no-runtime-check</code>, and the C++ "
        "programs with {compiler} <code>-O3 -march=native -std=gnu++20</code>.</p>"),
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


def build_sections(row, cases_dir):
    """For each comparison, the cases it draws, each with the Fix count and the reference count
    of every metric. A count the row left empty is `None`. `cases_dir` holds the cases, whose
    sources say which comparison a case belongs to."""
    sections = []
    for comparison in COMPARISONS:
        cases = set()
        for metric, _ in METRICS:
            tail = f"-{metric}-{comparison.language}"
            cases |= {name[: -len(tail)] for name in row if name.endswith(tail) and row[name]}
        cases = sorted(c for c in cases if comparison.draws(c, cases_dir / c))
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


def speedup(counts):
    """How many times faster the Fix program is: the reference's count over Fix's."""
    fix, reference = counts
    if not fix or not reference:
        return None
    return reference / fix


def geometric_mean(values):
    values = [v for v in values if v is not None]
    if not values:
        return None
    return math.exp(sum(math.log(v) for v in values) / len(values))


def format_speedup(value):
    return "&ndash;" if value is None else f"{value:.2f}&times;"


def format_count(value):
    return "&ndash;" if value is None else f"{value:,}"


def display_name(case):
    return case[len("cp_"):] if case.startswith("cp_") else case


def source_link(case, commit):
    """The case's directory on GitHub at the commit measured, where its Fix program sits beside
    its counterparts."""
    return f"{REPOSITORY}/tree/{commit}/benchmark/speedtest/cases/{case}"


def chart_rows(sections):
    """Every pair of a case and a comparison, the fastest for Fix first. A pair with no count of
    the sorting metric goes last."""
    rows = [(case, comparison, counts) for comparison, cases in sections for case, counts in cases]
    return sorted(rows, key=lambda r: -(speedup(r[2][SORT_METRIC]) or 0))


def tooltip(case, comparison, counts):
    """What hovering over a row says: the counts behind each bar."""
    lines = [f"{display_name(case)} vs {comparison.label}", comparison.fix_build]
    if case in CASE_NOTES:
        lines.append(CASE_NOTES[case])
    for metric, name in METRICS:
        fix, reference = counts[metric]
        s = speedup(counts[metric])
        verdict = ("not measured" if s is None else
                   f"Fix is {s:.2f}× faster" if s >= 1 else f"Fix is {1 / s:.2f}× slower")
        lines.append(f"{name}: {verdict}")
        if s is not None:
            lines.append(f"    Fix {fix:,} vs {comparison.label} {reference:,}")
    return "\n".join(lines)


# The geometry of the chart, in SVG user units. The SVG scales to the width of the page.
LABEL_WIDTH = 200
PLOT_WIDTH = 560
CHART_WIDTH = LABEL_WIDTH + PLOT_WIDTH + 24
BAR_HEIGHT = 11
ROW_HEIGHT = 2 * BAR_HEIGHT + 8
AXIS_HEIGHT = 44


def chart(rows, commit):
    """An SVG drawing a pair of bars per row: the log of each metric's speedup, with 1 at the
    centre. A bar to the right is a row where Fix took less."""
    logs = [abs(math.log2(s)) for _, _, counts in rows for metric, _ in METRICS
            if (s := speedup(counts[metric])) is not None]
    # The axis reaches the next power of two past the farthest bar, and at least to 2x either
    # way, so that a chart of near-equal rows does not magnify the noise.
    reach = max(1, math.ceil(max(logs, default=0) + 1e-9))
    scale = (PLOT_WIDTH / 2) / reach
    centre = LABEL_WIDTH + PLOT_WIDTH / 2
    bottom = AXIS_HEIGHT + ROW_HEIGHT * len(rows)
    height = bottom + 28
    out = [f'<svg class="chart" viewBox="0 0 {CHART_WIDTH} {height}" role="img" '
           f'aria-label="Speed of Fix relative to C, Rust and C++, per program">']
    out.append(f'<text class="side slower" x="{LABEL_WIDTH + 4}" y="14">&larr; Fix slower</text>')
    out.append(f'<text class="side faster" x="{LABEL_WIDTH + PLOT_WIDTH - 4}" y="14">'
               f'Fix faster &rarr;</text>')
    # The band that marks the row under the pointer comes first, so that the axis and the grid
    # stay drawn over it.
    for i in range(len(rows)):
        out.append(f'<rect class="band" data-band="{i}" x="0" y="{AXIS_HEIGHT + i * ROW_HEIGHT}" '
                   f'width="{CHART_WIDTH}" height="{ROW_HEIGHT}"/>')
    for k in range(-reach, reach + 1):
        x = centre + k * scale
        label = f"{2 ** k:g}&times;" if k >= 0 else f"1/{2 ** -k}"
        out.append(f'<line class="{"one" if k == 0 else "grid"}" x1="{x:.1f}" '
                   f'y1="{AXIS_HEIGHT - 10}" x2="{x:.1f}" y2="{bottom}"/>')
        for y in (AXIS_HEIGHT - 16, bottom + 18):
            out.append(f'<text class="tick" x="{x:.1f}" y="{y}">{label}</text>')
    for i, (case, comparison, counts) in enumerate(rows):
        top = AXIS_HEIGHT + i * ROW_HEIGHT
        tip = html.escape(tooltip(case, comparison, counts), quote=True)
        out.append(f'<g class="row" tabindex="0" data-band="{i}" data-tip="{tip}">'
                   f'<rect class="hit" x="0" y="{top}" width="{CHART_WIDTH}" '
                   f'height="{ROW_HEIGHT}"/>')
        middle = top + ROW_HEIGHT / 2 + 4
        out.append(f'<a class="case" href="{source_link(case, commit)}">'
                   f'<text x="{LABEL_WIDTH - 58}" y="{middle}">'
                   f'{html.escape(display_name(case))}</text></a>')
        out.append(f'<rect class="chip {comparison.key}" x="{LABEL_WIDTH - 50}" '
                   f'y="{middle - 12}" width="40" height="16" rx="3"/>'
                   f'<text class="chip-label" x="{LABEL_WIDTH - 30}" y="{middle}">'
                   f'{comparison.label}</text>')
        for j, (metric, _) in enumerate(METRICS):
            s = speedup(counts[metric])
            if s is None:
                continue
            width = math.log2(s) * scale
            x = centre + min(0, width)
            y = top + 4 + j * BAR_HEIGHT
            out.append(f'<rect class="bar {metric}" x="{x:.1f}" y="{y}" '
                       f'width="{max(abs(width), 1.5):.1f}" height="{BAR_HEIGHT - 2}"/>')
        out.append("</g>")
    out.append("</svg>")
    return "".join(out)


def table(comparison, rows, commit):
    head = "".join(f'<th colspan="3" class="group {metric}">{name}</th>' for metric, name in METRICS)
    sub = "".join(f"<th>Fix</th><th>{html.escape(comparison.label)}</th><th>Fix speedup</th>"
                  for _ in METRICS)
    body = []
    for case, counts in sorted(rows, key=lambda r: -(speedup(r[1][SORT_METRIC]) or 0)):
        cells = "".join(
            f"<td>{format_count(counts[m][0])}</td><td>{format_count(counts[m][1])}</td>"
            f"<td class=\"ratio\">{format_speedup(speedup(counts[m]))}</td>" for m, _ in METRICS)
        body.append(f'<tr><th scope="row"><a href="{source_link(case, commit)}">'
                    f'{html.escape(display_name(case))}</a></th>{cells}</tr>')
    means = "".join(f'<td colspan="2"></td><td class="ratio">'
                    f'{format_speedup(geometric_mean(speedup(c[m]) for _, c in rows))}</td>'
                    for m, _ in METRICS)
    return (f'<div class="table-wrap"><table><thead><tr><th rowspan="2">Program</th>{head}</tr>'
            f'<tr>{sub}</tr></thead><tbody>{"".join(body)}</tbody>'
            f'<tfoot><tr><th scope="row">Geometric mean</th>{means}</tr></tfoot></table></div>')


def compiler_name(compiler, row):
    """The compiler's name followed by the version the row recorded for it, if any."""
    version = row.get(f"version-{compiler}", "")
    return f"{compiler} {version}" if version else compiler


def section(comparison, rows, commit, row):
    conditions = comparison.conditions.replace(
        "{compiler}", html.escape(compiler_name(comparison.compiler, row)))
    return (f'<section><h3>{html.escape(comparison.title)}</h3>{conditions}'
            f'<details><summary>Show the counts ({len(rows)} programs)</summary>'
            f'{table(comparison, rows, commit)}</details></section>')


def render(row, cases_dir):
    commit = row["commit"]
    cpu = row.get("cpu", "")
    sections = build_sections(row, cases_dir)
    legend = "".join(f'<span class="key" tabindex="0" data-tip="{html.escape(METRIC_NOTES[metric])}">'
                     f'<span class="swatch {metric}"></span>{name}</span>'
                     for metric, name in METRICS)
    legend += "".join(f'<span class="key" tabindex="0" data-tip="{html.escape(c.summary.replace('{compiler}', compiler_name(c.compiler, row)))}">'
                      f'<span class="chip-key {c.key}">{c.label}</span>'
                      f'{html.escape(c.title)}</span>' for c, _ in sections)
    commit_link = f'<a href="{REPOSITORY}/commit/{commit}"><code>{commit[:7]}</code></a>'
    version = row.get("version", "")
    return TEMPLATE.format(
        build=f"Fix {html.escape(version)} ({commit_link})" if version else f"Fix {commit_link}",
        cpu=html.escape(cpu) if cpu else "an unrecorded processor",
        compilers=", ".join(html.escape(n) for n in
                            [f"Fix {version}" if version else "Fix"]
                            + [compiler_name(c, row) for c in ("gcc", "g++", "rustc")]),
        legend=legend, chart=chart(chart_rows(sections), commit),
        sections="".join(section(c, rows, commit, row) for c, rows in sections))


TEMPLATE = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Fix benchmarks</title>
<style>
:root {{
  --bg: #ffffff; --fg: #1f2328; --muted: #59636e; --rule: #d1d9e0; --grid: #eaeef2;
  --hover: #f3f6f9; --cycles: #0969da; --inst: #d4a72c;
  --c: #6e7781; --rust: #b35900; --acl: #8250df; --cpp: #1a7f37; --chip-fg: #ffffff;
  --tip-bg: #1f2328; --tip-fg: #f6f8fa;
}}
@media (prefers-color-scheme: dark) {{
  :root:not([data-theme="light"]) {{
    --bg: #0d1117; --fg: #e6edf3; --muted: #9198a1; --rule: #3d444d; --grid: #21262d;
    --hover: #161b22; --cycles: #4493f8; --inst: #d29922;
    --c: #8b949e; --rust: #db6d28; --acl: #a371f7; --cpp: #3fb950; --chip-fg: #0d1117;
    --tip-bg: #e6edf3; --tip-fg: #0d1117; color-scheme: dark;
  }}
}}
:root[data-theme="dark"] {{
  --bg: #0d1117; --fg: #e6edf3; --muted: #9198a1; --rule: #3d444d; --grid: #21262d;
  --hover: #161b22; --cycles: #4493f8; --inst: #d29922;
  --c: #8b949e; --rust: #db6d28; --acl: #a371f7; --cpp: #3fb950; --chip-fg: #0d1117;
  --tip-bg: #e6edf3; --tip-fg: #0d1117; color-scheme: dark;
}}
* {{ box-sizing: border-box; }}
body {{ margin: 0; background: var(--bg); color: var(--fg);
  font: 16px/1.55 -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif; }}
main {{ max-width: 960px; margin: 0 auto; padding-block: 28px 64px;
  padding-left: max(16px, env(safe-area-inset-left, 0px));
  padding-right: max(16px, env(safe-area-inset-right, 0px)); }}
header {{ display: flex; flex-wrap: wrap; align-items: baseline; gap: 4px 16px; }}
h1 {{ font-size: 1.7rem; margin: 0; text-wrap: balance; }}
.build {{ color: var(--muted); font-size: 0.95rem; }}
h2 {{ font-size: 1.3rem; margin: 56px 0 8px; padding-bottom: 6px; border-bottom: 1px solid var(--rule); }}
h3 {{ font-size: 1.05rem; margin: 28px 0 4px; }}
p {{ margin: 8px 0; }}
a {{ color: var(--cycles); }}
code {{ font: 0.88em ui-monospace, SFMono-Regular, Menlo, monospace; }}
.lead {{ color: var(--muted); margin-top: 4px; }}
.legend {{ display: flex; flex-wrap: wrap; gap: 8px 20px; margin: 14px 0 0; font-size: 0.88rem; color: var(--muted); }}
.key {{ display: inline-flex; align-items: center; gap: 6px;
  border-bottom: 1px dotted var(--rule); outline: none; }}
.key:hover, .key:focus, .key.active {{ color: var(--fg); border-bottom-color: var(--muted); }}
.swatch {{ width: 14px; height: 10px; display: inline-block; }}
.swatch.cycles {{ background: var(--cycles); }}
.swatch.inst {{ background: var(--inst); }}
.chip-key {{ color: var(--chip-fg); font-size: 0.75rem; font-weight: 600; padding: 0 6px; border-radius: 3px; }}
.chip-key.c {{ background: var(--c); }}
.chip-key.rust {{ background: var(--rust); }}
.chip-key.acl {{ background: var(--acl); }}
.chip-key.cpp {{ background: var(--cpp); }}
main {{ position: relative; }}
.chart {{ width: 100%; height: auto; margin: 8px 0; display: block; }}
.chart text {{ fill: var(--fg); font-size: 13px; }}
.chart .tick {{ text-anchor: middle; fill: var(--muted); font-size: 12px; }}
.chart .side {{ font-size: 13px; font-weight: 600; }}
.chart .side.slower {{ fill: var(--muted); }}
.chart .side.faster {{ text-anchor: end; fill: var(--fg); }}
.chart .case text {{ text-anchor: end; font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  fill: var(--cycles); }}
.chart .case:hover text, .chart .case:focus text {{ text-decoration: underline; }}
.chart .chip.c {{ fill: var(--c); }}
.chart .chip.rust {{ fill: var(--rust); }}
.chart .chip.acl {{ fill: var(--acl); }}
.chart .chip.cpp {{ fill: var(--cpp); }}
.chart .chip-label {{ fill: var(--chip-fg); text-anchor: middle; font-size: 11px; font-weight: 600; }}
.chart .grid {{ stroke: var(--grid); }}
.chart .one {{ stroke: var(--muted); stroke-width: 1.5; }}
.chart .bar.cycles {{ fill: var(--cycles); }}
.chart .bar.inst {{ fill: var(--inst); }}
.chart .hit, .chart .band {{ fill: transparent; }}
.chart .row {{ cursor: default; outline: none; }}
.chart .band.active {{ fill: var(--hover); }}
.tip {{ position: absolute; z-index: 1; pointer-events: none; background: var(--tip-bg); color: var(--tip-fg);
  font: 12px/1.5 ui-monospace, SFMono-Regular, Menlo, monospace; white-space: pre;
  padding: 8px 10px; border-radius: 6px; max-width: min(440px, calc(100vw - 32px)); }}
.tip.prose {{ font: 13px/1.5 -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
  white-space: normal; }}
.table-wrap {{ overflow-x: auto; margin-top: 8px; }}
details summary {{ cursor: pointer; color: var(--muted); font-size: 0.92rem; margin-top: 8px; }}
table {{ border-collapse: collapse; font-size: 0.85rem; font-variant-numeric: tabular-nums; width: 100%; }}
th, td {{ padding: 4px 8px; text-align: right; white-space: nowrap; border-bottom: 1px solid var(--grid); }}
thead th {{ border-bottom: 1px solid var(--rule); }}
th[scope="row"], thead th:first-child {{ text-align: left; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-weight: normal; }}
th.group.cycles {{ color: var(--cycles); }}
th.group.inst {{ color: var(--inst); }}
td.ratio {{ font-weight: 600; }}
tfoot th, tfoot td {{ border-top: 1px solid var(--rule); border-bottom: none; font-weight: 600; }}
.method p {{ color: var(--muted); font-size: 0.92rem; }}
@media (max-width: 600px) {{ .chart text {{ font-size: 15px; }} }}
</style>
</head>
<body>
<main>
<header><h1>How fast is Fix?</h1><span class="build">{build}</span></header>
<p class="lead">The speed of programs compiled by Fix, compared with the same programs written
in C, Rust and C++.</p>
<div class="legend">{legend}</div>
{chart}
<div class="tip" hidden></div>
<h2>Comparisons</h2>
{sections}
<h2>Measurement</h2>
<div class="method">
<p>Measured on {cpu}, with {compilers}.</p>
<p>All counts come from the CPU's hardware performance counters, read with <code>perf stat</code>
in user mode on a machine that runs nothing else. The cycle count measures how long a program
ran. It is the lowest of five measurements, because other activity can only add to it. The
instruction count is the number of instructions the program executed, which other activity does
not change.</p>
<p>The two can disagree. A program that waits on memory or mispredicts branches needs more cycles
per instruction. Where the code lands in memory can also change a cycle count by tens of percent
without changing any instruction.</p>
<p>The axis is logarithmic, so 2&times; faster and 2&times; slower are bars of the same
length.</p>
<p>The Fix programs are built with <code>fix build</code> at its default optimization level,
<code>-O max</code>. The programs and the
measurement scripts are in
<a href="{repository}/tree/main/benchmark/speedtest">benchmark/speedtest</a>. Each Fix program,
<code>main.fix</code>, sits next to its counterparts, <code>ref.c</code>, <code>ref.rs</code> or
<code>ref.cpp</code>.</p>
</div>
</main>
<script>
(() => {{
  // A tooltip for every element that carries `data-tip`: the rows of the chart, and the keys of
  // the legend. A mouse shows it while over the element; a tap or keyboard focus shows it until
  // the next one.
  const main = document.querySelector("main");
  const tip = main.querySelector(".tip");
  let active = [];
  const show = (target, x, y) => {{
    active.forEach((element) => element.classList.remove("active"));
    // A row of the chart lights its band, which is drawn beneath the axis.
    const band = target.dataset.band === undefined ? null
      : main.querySelector(`.band[data-band="${{target.dataset.band}}"]`);
    active = band ? [target, band] : [target];
    active.forEach((element) => element.classList.add("active"));
    tip.textContent = target.dataset.tip;
    tip.classList.toggle("prose", target.classList.contains("key"));
    tip.hidden = false;
    const box = main.getBoundingClientRect();
    const left = Math.min(x - box.left + 14, box.width - tip.offsetWidth);
    tip.style.left = Math.max(0, left) + "px";
    tip.style.top = (y - box.top + 16) + "px";
  }};
  const hide = () => {{
    active.forEach((element) => element.classList.remove("active"));
    active = [];
    tip.hidden = true;
  }};
  main.querySelectorAll("[data-tip]").forEach((target) => {{
    target.addEventListener("pointermove", (e) => {{
      if (e.pointerType === "mouse") show(target, e.clientX, e.clientY);
    }});
    target.addEventListener("pointerleave", (e) => {{ if (e.pointerType === "mouse") hide(); }});
    target.addEventListener("click", (e) => show(target, e.clientX, e.clientY));
    target.addEventListener("focus", () => {{
      const r = target.getBoundingClientRect();
      show(target, r.left + Math.min(r.width / 2, 40), r.bottom - 8);
    }});
    target.addEventListener("blur", hide);
  }});
  document.addEventListener("click", (e) => {{ if (!e.target.closest("[data-tip]")) hide(); }});
}})();
</script>
</body>
</html>
""".replace("{repository}", REPOSITORY)


def self_check():
    """Render a fixture log, so that a page bug shows on every invocation rather than on the
    published page."""
    with tempfile.TemporaryDirectory() as tmp:
        log = Path(tmp) / "log.csv"
        log.write_text(
            "commit,cpu,version,version-gcc,fib-inst,fib-cycles,fib-inst-unchecked,fib-cycles-unchecked,"
            "fib-inst-c,fib-cycles-c,fib-inst-rust,fib-cycles-rust,cp_uf-inst-unchecked,"
            "cp_uf-cycles-unchecked,cp_uf-inst-cpp,cp_uf-cycles-cpp,cp_sort-inst-unchecked,"
            "cp_sort-inst-cpp\n"
            "1111111111111111111111111111111111111111,Zen,1.6.0,13.3.0,100,200,90,180,90,90,50,,"
            "300,400,150,100,10,20\n"
            "2222222222222222222222222222222222222222(dirty),Zen,1.6.0,13.3.0,1,1,1,1,1,1,1,1,1,1,1,1,1,1\n",
            encoding="utf-8")
        header, body = read_log(log)
        cases = Path(tmp) / "cases"
        for case, source in [("cp_uf", "#include <atcoder/dsu>\n"),
                             ("cp_sort", "#include <algorithm>\n")]:
            (cases / case).mkdir(parents=True)
            (cases / case / "ref.cpp").write_text(source, encoding="utf-8")
        row = latest_row(header, body)
        sections = build_sections(row, cases)
        page = render(row, cases)
    # A dirty row names no commit, so the page is drawn from the row before it.
    assert row["commit"].startswith("1111"), row["commit"]
    by_key = {c.key: rows for c, rows in sections}
    # Fix is read from the unchecked build against C and C++, and from the checked build
    # against Rust.
    assert by_key["c"] == [("fib", {"cycles": (180, 90), "inst": (90, 90)})], by_key
    # A cycle count the run withheld leaves no speedup, and the case stays in the table.
    assert by_key["rust"] == [("fib", {"cycles": (200, None), "inst": (100, 50)})]
    # A C++ counterpart goes to the comparison with AtCoder Library when it calls the library,
    # and to the comparison with C++ otherwise.
    assert by_key["acl"] == [("cp_uf", {"cycles": (400, 100), "inst": (300, 150)})]
    assert by_key["cpp"] == [("cp_sort", {"cycles": (None, None), "inst": (10, 20)})]
    assert speedup(by_key["rust"][0][1]["cycles"]) is None
    # The chart puts the row where Fix is fastest by the sorting metric first.
    order = [(case, c.key) for case, c, _ in chart_rows(sections)]
    assert order == [("cp_sort", "cpp"), ("fib", "c"), ("fib", "rust"), ("cp_uf", "acl")], order
    assert abs(geometric_mean([2.0, 0.5]) - 1.0) < 1e-12
    # The slowest bar, a quarter, takes the axis to 1/4 on the left.
    assert "1/4" in page and "uf" in page
    assert "Fix 1.6.0 (" in page and "<code>1111111</code>" in page
    # A compiler's version reaches the page where the row recorded it, and its name alone where
    # the row did not.
    assert "built with gcc 13.3.0 <code>-O3" in page and "built with rustc <code>" in page
    assert "{compiler}" not in page
    assert f"/tree/{'1' * 40}/benchmark/speedtest/cases/cp_uf\"" in page


def main():
    self_check()
    here = Path(__file__).resolve().parent
    ap = argparse.ArgumentParser()
    ap.add_argument("--log", default=here / "log.csv", type=Path)
    ap.add_argument("--out", default=here / "comparison.html", type=Path)
    ap.add_argument("--cases", default=here / "cases", type=Path)
    args = ap.parse_args()
    header, body = read_log(args.log)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(render(latest_row(header, body), args.cases), encoding="utf-8")


if __name__ == "__main__":
    main()
