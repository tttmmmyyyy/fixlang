"""Keep a proof and the code it cites pointing at each other.

A proof under `dev-docs/proof/` cites code as `CODE <file>: <symbol>`, and each cited item carries a
`// PROOF: P1, P2 (<proof directory>)` comment naming the propositions that rest on it. The two
directions are generated from one source -- the citations in the proof -- so neither can drift from
the other unnoticed, and a reader editing an item sees at once whether a proof depends on it.

Beside them sits `citations.tsv`, which records the digest of each cited item's source text. A
digest that no longer matches says which propositions the code has moved out from under, which is
the question "did anything change since the proof was written" answered exactly rather than by
reading a transcription.

    python3 dev-docs/proof/proof_links.py            # check, exit non-zero on a finding
    python3 dev-docs/proof/proof_links.py --write    # rewrite the comments and the digests
"""

import glob
import hashlib
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# `CODE src/rc_ir/ownership.rs: origin_inner` and `CODE ...: Origin::identity`. The symbol stops at
# the first character a path cannot hold, so a citation that trails prose keeps only its symbol.
CITATION = re.compile(
    r"CODE\s+([A-Za-z0-9_/.]+\.(?:rs|fix))\s*:\s*`?"
    r"(impl\s+[A-Za-z_][A-Za-z0-9_]*\s+for\s+[A-Za-z_][A-Za-z0-9_]*"
    r"|[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*)"
)

# `| P1, P2 | `p10-leaves-and-units.md` | ... |` in a proof README's status table.
STATUS_ROW = re.compile(r"^\|\s*([^|]*P[0-9][^|]*?)\s*\|\s*`([^`]+\.md)`\s*\|", re.M)

PROOF_COMMENT = re.compile(r"^\s*// PROOF: ")


def propositions_of(text):
    """The propositions a status-table cell names, expanding `P8 - P14` into each of its members."""
    out = []
    for part in re.split(r"[、,]", text):
        part = part.strip().strip("*")
        span = re.fullmatch(r"P(\d+)\s*-\s*P?(\d+)", part)
        if span:
            out.extend(f"P{n}" for n in range(int(span.group(1)), int(span.group(2)) + 1))
            continue
        one = re.match(r"(P\d+)", part)
        if one:
            out.append(one.group(1))
    return out


def proof_dirs():
    """Every directory holding a proof: one with a README.md beside at least one `p*.md`."""
    for readme in sorted(glob.glob(os.path.join(REPO, "dev-docs/proof/*/*/README.md"))):
        directory = os.path.dirname(readme)
        if glob.glob(os.path.join(directory, "p*.md")):
            yield directory


def citations_of(directory):
    """Map each cited `(source file, symbol)` to the propositions that cite it, for one proof."""
    readme = open(os.path.join(directory, "README.md"), encoding="utf-8").read()
    by_file = {}
    for row in STATUS_ROW.finditer(readme):
        # A file may hold several rows -- one per group of propositions whose state differs -- and
        # every one of them names propositions the file proves.
        by_file.setdefault(row.group(2), []).extend(propositions_of(row.group(1)))
    cited = {}
    for path in sorted(glob.glob(os.path.join(directory, "p*.md"))):
        props = by_file.get(os.path.basename(path))
        if props is None:
            print(f"{os.path.relpath(path, REPO)}: no row in the README's status table")
            continue
        for hit in CITATION.finditer(open(path, encoding="utf-8").read()):
            cited.setdefault((hit.group(1), hit.group(2)), set()).update(props)
    return cited


def item_span(lines, symbol):
    """The half-open line range of a Rust item's definition, or `None` where it is not found.

    A one-part symbol names an item at any depth; a two-part `Owner::name` names one whose enclosing
    `impl`, `trait` or `mod` line carries the owner. The end is found by counting braces from the
    first one on or after the definition line, so an item without a body ends at its own line.
    """
    block = re.fullmatch(r"impl\s+(\w+)\s+for\s+(\w+)", symbol)
    if block:
        starts = re.compile(r"^\s*impl\b.*\b" + block.group(1) + r"\b.*\bfor\s+" + block.group(2) + r"\b")
        scope, owner = None, ""
    else:
        owner, _, name = symbol.rpartition("::")
        starts = re.compile(
            r"^\s*(?:pub(?:\([a-z()]+\))?\s+)?(?:async\s+|unsafe\s+|extern\s+\"[A-Za-z]+\"\s+)*"
            r"(?:fn|struct|enum|trait|type|union|const|static)\s+" + re.escape(name) + r"\b"
        )
        scope = re.compile(
            r"^\s*(?:pub(?:\([a-z()]+\))?\s+)?(?:impl|trait|mod)\b.*\b" + re.escape(owner) + r"\b"
        )
    enclosing = not owner
    for index, line in enumerate(lines):
        if owner and scope.search(line):
            enclosing = True
        if not starts.match(line):
            continue
        if not enclosing:
            continue
        depth, seen = 0, False
        for end in range(index, len(lines)):
            depth += lines[end].count("{") - lines[end].count("}")
            seen = seen or "{" in lines[end]
            if seen and depth <= 0:
                return index, end + 1
            if not seen and lines[end].rstrip().endswith(";"):
                return index, end + 1
        return index, len(lines)
    return None


def digest(lines, span):
    """The digest of an item's source text, with the `// PROOF:` lines inside it left out.

    An item's annotation, and the annotations of the items nested in it, move whenever the
    propositions do. Reading them into the digest would report the item as changed every time the
    proof's own bookkeeping changes, which is the one thing the digest must not do."""
    body = "".join(l for l in lines[span[0]:span[1]] if not PROOF_COMMENT.match(l))
    return hashlib.sha256(body.encode("utf-8")).hexdigest()[:12]


def comment_for(props, directory):
    ordered = sorted(props, key=lambda p: int(p[1:]))
    return f"// PROOF: {', '.join(ordered)} ({os.path.relpath(directory, REPO)})\n"


def main():
    write = "--write" in sys.argv[1:]
    findings = []
    for directory in proof_dirs():
        cited = citations_of(directory)
        rows, edits = [], {}
        for (source, symbol), props in sorted(cited.items()):
            path = os.path.join(REPO, source)
            if not os.path.exists(path):
                findings.append(f"{source}: cited by {sorted(props)} and does not exist")
                continue
            if not source.endswith(".rs"):
                rows.append((source, symbol, props, "-"))
                continue
            lines = open(path, encoding="utf-8").readlines()
            span = item_span(lines, symbol)
            if span is None:
                findings.append(f"{source}: `{symbol}` is cited and not found")
                continue
            rows.append((source, symbol, props, digest(lines, span)))
            at = annotation_line(lines, span[0])
            edits.setdefault(source, {}).setdefault(at, set()).update(props)
        comments = {
            source: {at: comment_for(props, directory) for at, props in items.items()}
            for source, items in edits.items()
        }
        if write:
            apply_comments(comments)
            write_table(directory, rows)
        else:
            findings.extend(check_comments(comments))
            findings.extend(check_table(directory, rows))
    for finding in findings:
        print(finding)
    return 1 if findings else 0


def annotation_line(lines, start):
    """Where an item's annotation belongs: above the attributes that head it, under its doc."""
    while start and lines[start - 1].lstrip().startswith("#["):
        start -= 1
    return start


def apply_comments(comments):
    """Put each item's comment where `annotation_line` says, replacing the one there."""
    for source, items in comments.items():
        path = os.path.join(REPO, source)
        lines = open(path, encoding="utf-8").readlines()
        for start, comment in sorted(items.items(), reverse=True):
            indent = re.match(r"\s*", lines[start]).group(0)
            if start and PROOF_COMMENT.match(lines[start - 1]):
                lines[start - 1] = indent + comment
            else:
                lines.insert(start, indent + comment)
        open(path, "w", encoding="utf-8").writelines(lines)


def check_comments(comments):
    """Every cited item carries its comment, and no other line claims one."""
    wanted = {}
    for source, items in comments.items():
        for start, comment in items.items():
            wanted.setdefault(source, {})[start] = comment.strip()
    for source in sorted(set(list(wanted) + rust_files_with_comments())):
        lines = open(os.path.join(REPO, source), encoding="utf-8").readlines()
        present = {i: line.strip() for i, line in enumerate(lines) if PROOF_COMMENT.match(line)}
        for start, comment in sorted(wanted.get(source, {}).items()):
            if present.pop(start - 1, None) != comment:
                yield f"{source}:{start + 1}: the item's `// PROOF:` comment is missing or stale"
        for index in sorted(present):
            yield f"{source}:{index + 1}: a `// PROOF:` comment no proof citation asks for"


def rust_files_with_comments():
    out = []
    for path in glob.glob(os.path.join(REPO, "src/**/*.rs"), recursive=True):
        with open(path, encoding="utf-8") as handle:
            if any(PROOF_COMMENT.match(line) for line in handle):
                out.append(os.path.relpath(path, REPO))
    return out


def table_path(directory):
    return os.path.join(directory, "citations.tsv")


def write_table(directory, rows):
    with open(table_path(directory), "w", encoding="utf-8") as handle:
        handle.write("# generated by dev-docs/proof/proof_links.py -- do not edit\n")
        handle.write("file\tsymbol\tpropositions\tdigest\n")
        for source, symbol, props, dig in rows:
            ordered = ",".join(sorted(props, key=lambda p: int(p[1:])))
            handle.write(f"{source}\t{symbol}\t{ordered}\t{dig}\n")


def check_table(directory, rows):
    path = table_path(directory)
    if not os.path.exists(path):
        yield f"{os.path.relpath(path, REPO)}: missing; run with --write"
        return
    recorded = {}
    for line in open(path, encoding="utf-8").readlines()[2:]:
        source, symbol, props, dig = line.rstrip("\n").split("\t")
        recorded[(source, symbol)] = dig
    for source, symbol, props, dig in rows:
        was = recorded.get((source, symbol))
        if was is None:
            yield f"{source}: `{symbol}` is cited and absent from citations.tsv"
        elif was != dig:
            names = ",".join(sorted(props, key=lambda p: int(p[1:])))
            yield f"{source}: `{symbol}` changed since the proof was written; re-verify {names}"


if __name__ == "__main__":
    sys.exit(main())
