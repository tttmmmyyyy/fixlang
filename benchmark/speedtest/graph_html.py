"""Render the benchmark log as a single self-contained HTML page.

Reads `log.csv` (one row per measured commit, one column per case and metric) and
`history.md` (prose for the commits whose numbers moved), and writes a page that
plots every case per metric. The page carries its data inline and loads nothing
from the network, so it works from a file:// URL and from GitHub Pages alike.

    python3 graph_html.py [--log log.csv] [--history history.md] [--out graph.html]
"""

import argparse
import csv
import html
import json
import re
from pathlib import Path

try:
    import markdown as _markdown
except ImportError:
    _markdown = None

# Metric suffix in `log.csv` -> how the page presents it.
#
# `ratio` divides every point by the series' first measured value, which is what makes
# cases of wildly different absolute cost readable on one axis. `absolute` keeps the raw
# count: a split-access column is interesting precisely when it collapses by orders of
# magnitude, and zero -- the value a fixed case reaches -- has no ratio.
METRICS = [
    ("inst", "Cachegrind instructions",
     "Instructions executed (Ir), from cachegrind's simulation. The same program and input give the "
     "same number on any machine, whatever else it is doing.", "ratio", "cachegrind"),
    ("mem", "Cachegrind memory",
     "Weighted memory-access estimate from cachegrind's cache model (l1 + 5*l3 + 35*ram).",
     "ratio", "cachegrind"),
    ("splits", "perf splits",
     "Loads and stores that crossed a cache-line boundary, from the hardware counters. Cachegrind's "
     "model has no notion of these, and they cost real time; the count is deterministic and reaches "
     "zero once the data is aligned, so it is plotted as an absolute count.", "absolute", "perf"),
]


def parse_history(path):
    """Map each commit hash in `history.md` to the HTML of its entry.

    An entry runs from a `## <hash>` heading to the next one.
    """
    if not path.exists():
        return {}
    entries = {}
    current_hash, buf = None, []

    def flush():
        if current_hash is None:
            return
        text = "\n".join(buf).strip()
        entries[current_hash] = render_markdown(text) if text else ""

    for line in path.read_text(encoding="utf-8").splitlines():
        m = re.match(r"^##\s+([0-9a-f]{7,40})\s*$", line)
        if m:
            flush()
            current_hash, buf = m.group(1), []
        elif current_hash is not None:
            buf.append(line)
    flush()
    return entries


def render_markdown(text):
    if _markdown is not None:
        return _markdown.markdown(text, extensions=["tables"])
    # Enough of Markdown for the prose these entries use.
    out = html.escape(text)
    out = re.sub(r"`([^`]+)`", r"<code>\1</code>", out)
    out = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", out)
    return "".join(f"<p>{p.strip()}</p>" for p in out.split("\n\n") if p.strip())


def read_log(path):
    with path.open(encoding="utf-8") as f:
        rows = [r for r in csv.reader(f) if any(c.strip() for c in r)]
    header, body = rows[0], rows[1:]
    return header, body


def build_data(log_path, history_path, latest_n):
    header, body = read_log(log_path)
    body = body[-latest_n:]
    history = parse_history(history_path)

    index = {name.strip(): i for i, name in enumerate(header)}
    cpu_col = index.get("cpu")

    commits = []
    for row in body:
        raw = row[0].strip()
        dirty = raw.endswith("(dirty)")
        h = raw[: -len("(dirty)")] if dirty else raw
        entry = history.get(h)
        if entry is None:  # `history.md` may key an entry by a short hash.
            entry = next((v for k, v in history.items() if h.startswith(k) or k.startswith(h)), None)
        cpu = row[cpu_col].strip() if cpu_col is not None and cpu_col < len(row) else ""
        commits.append({"hash": h, "short": h[:8], "dirty": dirty, "history": entry, "cpu": cpu})

    # Split each "<case>-<metric>" column apart.
    metrics = {}
    for suffix, label, note, kind, source in METRICS:
        series = {}
        for name, i in index.items():
            if not name.endswith("-" + suffix):
                continue
            case = name[: -(len(suffix) + 1)]
            values = []
            for row in body:
                cell = row[i].strip() if i < len(row) else ""
                values.append(int(cell) if cell.isdigit() else None)
            if any(v is not None for v in values):
                series[case] = values
        if series:
            metrics[suffix] = {"label": label, "note": note, "kind": kind,
                               "source": source, "series": series}

    return {"commits": commits, "metrics": metrics}


def main():
    here = Path(__file__).resolve().parent
    ap = argparse.ArgumentParser()
    ap.add_argument("--log", default=here / "log.csv", type=Path)
    ap.add_argument("--history", default=here / "history.md", type=Path)
    ap.add_argument("--out", default=here / "graph.html", type=Path)
    ap.add_argument("--latest-n", default=40, type=int)
    args = ap.parse_args()

    data = build_data(args.log, args.history, args.latest_n)
    page = TEMPLATE.replace("__DATA__", json.dumps(data, ensure_ascii=False))
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(page, encoding="utf-8")
    n_cases = max((len(m["series"]) for m in data["metrics"].values()), default=0)
    print(f"{args.out}: {len(data['commits'])} commits, {n_cases} cases, "
          f"{len(data['metrics'])} metrics")


TEMPLATE = r"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Fix benchmark history</title>
<style>
/* One fixed palette, so the page looks the same whatever the browser or OS theme is. */
:root {
  color-scheme: light;
  --bg: #ffffff; --fg: #1b1b1f; --muted: #6b6b76; --grid: #e8e8ee;
  --panel: #f7f7fa; --border: #dcdce4; --accent: #2f6fd0;
}
* { box-sizing: border-box; }
body { margin: 0; background: var(--bg); color: var(--fg);
       font: 15px/1.6 system-ui, "Helvetica Neue", Arial, sans-serif; }
header { padding: 20px 24px 0; }
h1 { font-size: 20px; margin: 0 0 12px; }
.tabs { display: flex; flex-wrap: wrap; gap: 6px; }
.tab { border: 1px solid var(--border); background: var(--panel); color: var(--fg);
       padding: 6px 14px; border-radius: 999px; cursor: pointer; font-size: 13px; }
.tab[aria-selected="true"] { background: var(--accent); border-color: var(--accent); color: #fff; }
.note { color: var(--muted); font-size: 12.5px; margin: 8px 0 0; max-width: 90ch; }
main { display: grid; grid-template-columns: minmax(0, 1fr) 340px; gap: 20px; padding: 12px 24px 32px; }
@media (max-width: 1000px) { main { grid-template-columns: minmax(0, 1fr); } }
.chart-wrap { position: relative; min-width: 0; }
svg { width: 100%; height: auto; display: block; overflow: visible; }
svg text { fill: var(--muted); font-size: 11px; }
.grid line { stroke: var(--grid); stroke-width: 1; }
.serie { fill: none; stroke-width: 1.7; opacity: .85; }
.serie.dim { opacity: .1; }
.serie.hot { stroke-width: 3.2; opacity: 1; }
.hit { fill: none; stroke: transparent; stroke-width: 14; cursor: pointer; }
.xtick { cursor: pointer; }
.xtick text { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 10.5px; }
.xtick.has-history text { fill: var(--fg); }
.xtick.on text { fill: var(--accent); font-weight: 700; }
#tip { position: absolute; pointer-events: none; background: var(--panel); color: var(--fg);
       border: 1px solid var(--border); border-radius: 8px; padding: 7px 10px; font-size: 12.5px;
       box-shadow: 0 4px 14px rgba(0,0,0,.14); opacity: 0; transition: opacity .08s; max-width: 320px; }
aside { border-left: 1px solid var(--border); padding-left: 18px; min-width: 0; }
@media (max-width: 1000px) { aside { border-left: 0; border-top: 1px solid var(--border);
                                     padding: 16px 0 0; } }
aside h2 { font-size: 13px; text-transform: uppercase; letter-spacing: .06em;
           color: var(--muted); margin: 0 0 8px; }
#history { font-size: 13.5px; }
#history :is(h2, h3) { font-size: 14px; margin: 14px 0 6px; }
#history code { background: var(--grid); padding: 1px 4px; border-radius: 4px;
                font-size: 12px; word-break: break-all; }
#history table { border-collapse: collapse; font-size: 12px; margin: 8px 0; }
#history :is(th, td) { border: 1px solid var(--border); padding: 3px 7px; }
#history .commit { font-family: ui-monospace, monospace; color: var(--muted); font-size: 12px; }
#history .placeholder { color: var(--muted); }
.legend { display: flex; flex-wrap: wrap; gap: 3px 12px; margin-top: 18px; }
.legend button { display: inline-flex; align-items: center; gap: 5px; background: none; border: 0;
                 color: var(--fg); cursor: pointer; font-size: 12.5px; padding: 1px 0;
                 font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.legend button:hover { color: var(--accent); }
.legend button.faded { color: var(--muted); opacity: .45; }
.legend .swatch { width: 11px; height: 11px; border-radius: 3px; flex: none; }
</style>
</head>
<body>
<header>
  <h1>Fix benchmark history</h1>
  <div class="tabs" id="tabs" role="tablist"></div>
  <p class="note" id="note"></p>
</header>
<main>
  <div class="chart-wrap">
    <svg id="chart" viewBox="0 0 960 460" preserveAspectRatio="xMidYMid meet"></svg>
    <div id="tip"></div>
    <div class="legend" id="legend"></div>
  </div>
  <aside>
    <h2>Commit notes</h2>
    <div id="history"><p class="placeholder">Click a commit on the axis.</p></div>
  </aside>
</main>
<script>
const DATA = __DATA__;
const W = 960, H = 460, PAD = { t: 14, r: 18, b: 74, l: 54 };
const PLOT_W = W - PAD.l - PAD.r, PLOT_H = H - PAD.t - PAD.b;
const SVG_NS = "http://www.w3.org/2000/svg";

const el = (name, attrs = {}, parent = null) => {
  const n = document.createElementNS(SVG_NS, name);
  for (const [k, v] of Object.entries(attrs)) n.setAttribute(k, v);
  if (parent) parent.appendChild(n);
  return n;
};
const color = (i) => `hsl(${(i * 47) % 360} 62% ${i % 2 ? 42 : 58}%)`;
const fmt = (v) => v >= 1e9 ? (v / 1e9).toFixed(2) + "G"
                 : v >= 1e6 ? (v / 1e6).toFixed(2) + "M"
                 : v >= 1e3 ? (v / 1e3).toFixed(1) + "k" : String(Math.round(v));

let metricKey = Object.keys(DATA.metrics)[0];
let onlyCase = null;  // the single case shown, or null for all of them
let selected = null;  // the commit whose notes the pane shows

function seriesOf(metric) {
  // Ratio metrics divide by the first measured point; absolute ones keep the count.
  const out = [];
  const names = Object.keys(metric.series).sort();
  names.forEach((name, i) => {
    const raw = metric.series[name];
    let base = 1;
    if (metric.kind === "ratio") {
      const first = raw.find((v) => v !== null && v !== 0);
      if (!first) return;
      base = first;
    }
    out.push({
      name, color: color(i), raw, base,
      values: raw.map((v) => (v === null ? null : v / base)),
    });
  });
  return out;
}

const visible = (s) => onlyCase === null || s.name === onlyCase;

function scaleY(series, kind) {
  let lo = Infinity, hi = -Infinity;
  for (const s of series) {
    if (!visible(s)) continue;
    for (const v of s.values) if (v !== null && v > 0) { lo = Math.min(lo, v); hi = Math.max(hi, v); }
  }
  if (!isFinite(lo)) { lo = 1; hi = 1; }
  if (kind === "ratio") { lo = Math.min(lo, 0.8); hi = Math.max(hi, 1.25); }
  const logPad = Math.max((Math.log10(hi) - Math.log10(lo)) * 0.12, 0.05);
  const logLo = Math.log10(lo) - logPad, logHi = Math.log10(hi) + logPad;
  // A zero sits below the axis; park it on the floor so the line stays continuous.
  return { toY: (v) =>
             PAD.t + PLOT_H * (1 - (Math.log10(Math.max(v, 10 ** logLo)) - logLo) / (logHi - logLo)),
           logLo, logHi };
}

function render() {
  const metric = DATA.metrics[metricKey];
  const svg = document.getElementById("chart");
  svg.textContent = "";
  const series = seriesOf(metric);
  const n = DATA.commits.length;
  const xAt = (i) => PAD.l + (n === 1 ? PLOT_W / 2 : (PLOT_W * i) / (n - 1));
  const { toY, logLo, logHi } = scaleY(series, metric.kind);

  const grid = el("g", { class: "grid" }, svg);
  const axis = el("g", { class: "axis" }, svg);
  const step = (logHi - logLo) > 2.5 ? 1 : (logHi - logLo) > 1 ? 0.5 : 0.25;
  for (let e = Math.ceil(logLo / step) * step; e <= logHi; e += step) {
    const v = 10 ** e, y = toY(v);
    if (y < PAD.t - 1 || y > PAD.t + PLOT_H + 1) continue;
    el("line", { x1: PAD.l, y1: y, x2: PAD.l + PLOT_W, y2: y }, grid);
    const label = metric.kind === "ratio"
      ? (v >= 10 ? v.toFixed(0) + "x" : v.toFixed(2) + "x")
      : fmt(v);
    el("text", { x: PAD.l - 8, y: y + 3.5, "text-anchor": "end" }, axis).textContent = label;
  }

  const lines = el("g", {}, svg);
  for (const s of series) {
    if (!visible(s)) continue;
    const pts = [];
    s.values.forEach((v, i) => { if (v !== null) pts.push(`${xAt(i)},${toY(v)}`); });
    if (!pts.length) continue;
    const d = "M" + pts.join("L");
    el("path", { class: "serie", stroke: s.color, d, "data-name": s.name }, lines);
    const hit = el("path", { class: "hit", d, "data-name": s.name }, lines);
    hit.addEventListener("mousemove", (ev) => hoverPoint(ev, s, xAt));
    hit.addEventListener("mouseleave", clearHover);
    // Clicking a line selects the commit it was clicked over, as clicking its tick would.
    hit.addEventListener("click", (ev) => {
      const i = nearestIndex(ev, s, xAt);
      selected = i;
      render();
      showHistory(i);
      highlight(s.name);
    });
  }

  const ticks = el("g", {}, svg);
  DATA.commits.forEach((c, i) => {
    const g = el("g", {
      class: "xtick" + (c.history ? " has-history" : "") + (selected === i ? " on" : ""),
    }, ticks);
    const x = xAt(i), y = PAD.t + PLOT_H;
    el("line", { x1: x, y1: y, x2: x, y2: y + 5, stroke: "var(--grid)" }, g);
    const t = el("text", {
      x: 0, y: 0, transform: `translate(${x} ${y + 10}) rotate(58)`, "text-anchor": "start",
    }, g);
    t.textContent = c.short + (c.dirty ? "*" : "");
    // A wide invisible target, so the small rotated label is easy to hit.
    el("rect", { x: x - 7, y: y, width: 14, height: PAD.b - 6, fill: "transparent" }, g);
    // Selection is on click alone: hovering would change the pane out from under the
    // pointer on its way over to read it.
    g.addEventListener("click", () => { selected = i; render(); showHistory(i); });
  });

  el("line", { x1: PAD.l, y1: PAD.t + PLOT_H, x2: PAD.l + PLOT_W, y2: PAD.t + PLOT_H,
               stroke: "var(--border)" }, svg);
  const note = document.getElementById("note");
  note.textContent = metric.note;
  // A count read from the hardware belongs to the processor that read it.
  if (metric.source === "perf") {
    const cpus = [...new Set(DATA.commits.map((c) => c.cpu).filter(Boolean))];
    if (cpus.length > 1) {
      note.textContent += ` Measured on more than one processor (${cpus.join("; ")}); `
        + "counts from different ones do not belong on the same axis.";
    }
  }
  renderLegend(series);
}

function highlight(name) {
  document.querySelectorAll(".serie").forEach((p) => {
    p.classList.toggle("dim", name !== null && p.dataset.name !== name);
    p.classList.toggle("hot", name !== null && p.dataset.name === name);
  });
}

// The commit whose column the pointer is nearest, among those the series measured.
function nearestIndex(ev, s, xAt) {
  const box = document.getElementById("chart").getBoundingClientRect();
  const rel = ((ev.clientX - box.left) / box.width) * W;
  let best = 0, bestD = Infinity;
  DATA.commits.forEach((_, i) => {
    const d = Math.abs(xAt(i) - rel);
    if (d < bestD && s.values[i] !== null) { bestD = d; best = i; }
  });
  return best;
}

function hoverPoint(ev, s, xAt) {
  const box = document.getElementById("chart").getBoundingClientRect();
  const best = nearestIndex(ev, s, xAt);
  const raw = s.raw[best], prev = [...s.raw.slice(0, best)].reverse().find((v) => v !== null);
  const delta = prev ? ((raw - prev) / prev) * 100 : null;
  highlight(s.name);
  const tip = document.getElementById("tip");
  tip.innerHTML = `<strong>${s.name}</strong><br>${DATA.commits[best].short}: ${raw.toLocaleString()}`
    + (delta === null ? "" : `<br>${delta >= 0 ? "+" : ""}${delta.toFixed(2)}% vs previous`);
  tip.style.opacity = 1;
  tip.style.left = Math.min(ev.clientX - box.left + 14, box.width - 200) + "px";
  tip.style.top = ev.clientY - box.top - 10 + "px";
}

function clearHover() {
  highlight(null);
  document.getElementById("tip").style.opacity = 0;
}

function showHistory(i) {
  const c = DATA.commits[i];
  document.getElementById("history").innerHTML =
    `<p class="commit">${c.hash}${c.dirty ? " (dirty)" : ""}${c.cpu ? "<br>" + c.cpu : ""}</p>`
    + (c.history || `<p class="placeholder">No note recorded for this commit.</p>`);
}

function renderLegend(series) {
  const box = document.getElementById("legend");
  box.textContent = "";
  // Clicking a case shows it on its own; clicking it again brings the rest back.
  for (const s of series) {
    const b = document.createElement("button");
    b.className = onlyCase !== null && onlyCase !== s.name ? "faded" : "";
    b.innerHTML = `<span class="swatch" style="background:${s.color}"></span>${s.name}`;
    b.addEventListener("mouseenter", () => highlight(s.name));
    b.addEventListener("mouseleave", () => highlight(null));
    b.addEventListener("click", () => {
      onlyCase = onlyCase === s.name ? null : s.name;
      render();
    });
    box.appendChild(b);
  }
}

function renderTabs() {
  const box = document.getElementById("tabs");
  box.textContent = "";
  for (const [key, m] of Object.entries(DATA.metrics)) {
    const b = document.createElement("button");
    b.className = "tab";
    b.textContent = m.label;
    b.setAttribute("role", "tab");
    b.setAttribute("aria-selected", key === metricKey);
    b.addEventListener("click", () => { metricKey = key; renderTabs(); render(); });
    box.appendChild(b);
  }
}

renderTabs();
render();
</script>
</body>
</html>
"""


main()
