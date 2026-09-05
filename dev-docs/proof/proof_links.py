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

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import proof_index
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# `CODE src/rc_ir/ownership.rs: origin, origin_inner`. One citation names a file and then a
# comma-separated list of that file's symbols, so the head and the symbol are matched separately and
# the list is walked by `citations_in`.
CITATION_HEAD = re.compile(r"CODE\s+([A-Za-z0-9_/.]+\.(?:rs|fix|pest))\s*:\s*")
CITATION_SYMBOL = re.compile(
    r"\s*`?(impl\s+[A-Za-z_][A-Za-z0-9_]*\s+for\s+[A-Za-z_][A-Za-z0-9_]*"
    r"|[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*)`?"
)

# What ends a citation's list of symbols: the next citation of any kind. A `BY` line runs the groups
# together with the same comma the list uses, so a name shaped like a label closes the list rather
# than being read as one more symbol of the file.
CITATION_LABEL = re.compile(
    r"(?:[A-Z]\d+[a-z]?|p\d+|CODE|DEF|EXT|BY|PROVE|ASSUME|QED|CASE|DEFINE)\Z"
)


def citations_in(text):
    """Every `(source file, symbol)` a proof's text cites.

    A citation names one file and as many of its symbols as the step relies on. Reading only the
    first would leave the rest of them out of both directions of the link -- the cited item would
    carry no `// PROOF:` comment, and `citations.tsv` would not notice it changing."""
    for head in CITATION_HEAD.finditer(text):
        source, at = head.group(1), head.end()
        while True:
            hit = CITATION_SYMBOL.match(text, at)
            if hit is None:
                break
            symbol, at = hit.group(1), hit.end()
            if CITATION_LABEL.match(symbol):
                break
            yield source, symbol
            at = skip_note(text, at)
            if not text.startswith(",", at):
                break
            at += 1


def skip_note(text, at):
    """The position past a parenthesised note following a cited symbol, or `at` where none stands.

    A citation may say which part of an item it relies on -- `RcVar (`ty` の doc)`. The note sits
    between the symbol and the comma that introduces the next symbol, so a reader that looks for the
    comma where the symbol ends stops at the first annotated symbol and drops every symbol after it,
    silently: a dropped symbol carries no `// PROOF:` comment and is never checked to exist."""
    start = at
    while start < len(text) and text[start] in " \t":
        start += 1
    if start >= len(text) or text[start] != "(":
        return at
    depth = 0
    for index in range(start, len(text)):
        if text[index] == "\n" and text[index - 1] == "\n":
            return at
        depth += (text[index] == "(") - (text[index] == ")")
        if depth == 0:
            return index + 1
    return at

# `| P1, P2 | `p10-leaves-and-units.md` | ... |` in a proof README's status table. The main theorem's
# row names it `T`, so a cell holding that alone counts as well.
STATUS_ROW = re.compile(
    r"^\|\s*((?:[^|]*P[0-9A-Za-z-][^|]*?)|T)\s*\|\s*`([^`]+\.md)`\s*\|", re.M
)

PROOF_COMMENT = re.compile(r"^\s*// PROOF: ")


def propositions_of(text):
    """The propositions a status-table cell names, expanding `P8 - P14` into each of its members.

    A cell may also carry a name that is not a numbered proposition -- the main theorem's `T`, or a
    label like `(P-insert)` for a file whose obligation is an assumption rather than a proposition.
    Such a name is kept as it stands, so that the items only that file cites still get a comment
    saying which part of the proof rests on them."""
    out = []
    for part in re.split(r"[、,]", text):
        part = part.strip().strip("*")
        span = re.fullmatch(r"P(\d+)\s*-\s*P?(\d+)", part)
        if span:
            out.extend(f"P{n}" for n in range(int(span.group(1)), int(span.group(2)) + 1))
            continue
        # The letter is part of the name: `P7a` and `P7e` are different propositions in different
        # files, and dropping it would give six of them one comment.
        one = re.match(r"(P\d+[a-z]?)", part)
        if one:
            out.append(one.group(1))
            continue
        # A bare name (the main theorem's `T`) or a parenthesised label of more than one letter
        # (`(P-insert)`). One letter in parentheses is a clause of the proposition beside it --
        # the `(b)` of `P5 (a), (b)` -- and names no file of its own.
        if re.fullmatch(r"[A-Za-z][A-Za-z0-9_-]*", part) or re.fullmatch(
            r"\([A-Za-z][A-Za-z0-9_-]+\)", part
        ):
            out.append(part)
    return out


def proof_dirs():
    """Every directory holding a proof: one with a README.md beside at least one `p*.md`."""
    for readme in sorted(glob.glob(os.path.join(REPO, "dev-docs/proof/*/*/README.md"))):
        directory = os.path.dirname(readme)
        if glob.glob(os.path.join(directory, "p*.md")):
            yield directory


# What a `// PROOF:` comment names when the citation comes from the README rather than from a
# proof of some proposition -- the definitions and the assumptions the whole proof is framed in.
FRAME = "D/A"


def citations_of(directory):
    """Map each cited `(source file, symbol)` to the propositions that cite it, for one proof.

    The README cites code too, in its definitions and its assumptions, and those citations carry the
    same weight: an item a definition rests on has moved out from under the whole proof when it
    changes. They are collected under `FRAME`."""
    readme = open(os.path.join(directory, "README.md"), encoding="utf-8").read()
    # Which propositions a file proves is read off the file itself -- its title names the ones
    # whose statement lives elsewhere, and its headings name the ones it states. A hand-kept table
    # saying the same thing goes stale, and the one that stood here carried ranges (`P8 - P14`),
    # which drop every lettered proposition inserted between their endpoints.
    by_file = {}
    for path in proof_index.documents(directory):
        name = os.path.basename(path)
        if name == "README.md":
            continue
        found, _ = proof_index.items_in(path)
        # The file's title names the propositions it proves; its headings name the ones local to it.
        # Only the first are meaningful to someone reading the code, since a local name means
        # nothing outside its own file.
        by_file[name] = [item["name"] for item in found if item["name"] and item["line"] == 1]
    cited = {}
    for path in sorted(glob.glob(os.path.join(directory, "*.md"))):
        name = os.path.basename(path)
        if name == "README.md":
            continue
        props = by_file.get(name)
        if not props and name.startswith("p"):
            print(f"{os.path.relpath(path, REPO)}: names no proposition it proves")
            continue
        # A document that proves no proposition discharges an assumption instead, so its citations
        # carry the frame's weight: the enumeration behind an assumption has moved out from under
        # the whole proof when the code it reads changes.
        for citation in citations_in(open(path, encoding="utf-8").read()):
            cited.setdefault(citation, set()).update(props or {FRAME})
    for citation in citations_in(readme):
        cited.setdefault(citation, set()).add(FRAME)
    return cited


# How an assumption's discharger states where an effect occurs: `SCAN <root> `<literal>`` followed
# by one `= <file>: <symbol>` line per member. The tool runs the search and compares.
#
# **This belongs to the discharge of an assumption, not to a step.** A step that enumerates the
# code is doing an assumption's work inline, where nobody names a discharger and nothing checks the
# enumeration. Measured, three files carried that shape and it was the largest family round 8 found.
#
# **The literal is what makes it runnable.** A predicate in prose -- "the calls to `set_refcnt_state`"
# -- cannot be run, so the enumeration it heads is checked by nobody. Writing `.set_refcnt_state(`
# says the same thing in a form the tool executes.
SCAN_HEAD = re.compile(r"^\s*SCAN\s+(\S+)\s+`([^`]+)`\s*$", re.M)
# A member may carry a note after `--`: the search is a lexical over-approximation, so the
# discharger says of each hit what it is. The note is not compared; its absence is not an error.
SCAN_MEMBER = re.compile(r"^\s*=\s*([A-Za-z0-9_/.]+)\s*:\s*(\S+)\s*(?:--.*)?$")
ITEM_HEAD = re.compile(
    r"^(\s*)(?:pub(?:\([a-z()]+\))?\s+)?(?:async\s+|unsafe\s+|extern\s+\"[A-Za-z]+\"\s+)*"
    r"(?:fn|struct|enum|trait|type|union|const|static)\s+(\w+)")
IMPL_HEAD = re.compile(r"^\s*(?:pub(?:\([a-z()]+\))?\s+)?impl\b[^{]*?(?:for\s+)?(\w+)\s*\{")


def item_at(lines, index):
    """The item a line belongs to, as the corpus writes it: `Owner::name`, or `name`.

    A closure claim names where each occurrence sits, so the scan has to answer in the same
    vocabulary the proof uses."""
    name = owner = None
    for at in range(index, -1, -1):
        if name is None:
            head = ITEM_HEAD.match(lines[at])
            if head:
                name = head.group(2)
                continue
        else:
            block = IMPL_HEAD.match(lines[at])
            if block:
                owner = block.group(1)
                break
            if ITEM_HEAD.match(lines[at]) and not lines[at].startswith((" ", "\t")):
                break
    if name is None:
        return None
    return f"{owner}::{name}" if owner else name


def scan_hits(root, literal):
    """Every item of `root` whose text contains `literal`, and whether it sits under `#[cfg(test)]`."""
    found = []
    for directory, _, files in os.walk(root):
        for file_name in sorted(files):
            if not file_name.endswith((".rs", ".fix", ".pest")):
                continue
            path = os.path.join(directory, file_name)
            with open(path, encoding="utf-8", errors="ignore") as source:
                lines = source.read().split("\n")
            # **`#[cfg(test)]` はその直後の項目にしか掛からない。** そこから末尾までをテストと
            # 見ると、その後ろに置かれた製品の項目まで落ちる -- 実測で、1 ファイルの 352 行目の
            # 属性が、1,403 行目の製品の関数をテストにしていた。
            tests = []
            for index, line in enumerate(lines):
                if "#[cfg(test)]" in line:
                    tests.append(item_body(lines, index + 1))
            for index, line in enumerate(lines):
                if literal not in line:
                    continue
                name = item_at(lines, index)
                if name:
                    in_test = any(start <= index < stop for start, stop in tests)
                    found.append((path, name, in_test))
    return found


def scans_in(path):
    """Every `SCAN` a proof file states, with the members it lists."""
    lines = open(path, encoding="utf-8").read().split("\n")
    out = []
    for index, line in enumerate(lines):
        head = SCAN_HEAD.match(line)
        if not head:
            continue
        members, at = [], index + 1
        while at < len(lines):
            member = SCAN_MEMBER.match(lines[at])
            if not member:
                break
            members.append((member.group(1), member.group(2)))
            at += 1
        out.append((index + 1, head.group(1), head.group(2), members))
    return out


def check_scans(directory):
    """Compare what each `SCAN` lists with what the search finds. Returns the differences."""
    problems = []
    for path in sorted(glob.glob(os.path.join(directory, "*.md"))):
        with open(path, encoding="utf-8") as handle:
            if "<!--not-a-proof-->" in handle.read(400):
                continue
        for line, root, literal, members in scans_in(path):
            hits = scan_hits(os.path.join(REPO, root), literal)
            found = {(os.path.relpath(where, REPO), name) for where, name, is_test in hits if not is_test}
            listed = set(members)
            for missing in sorted(found - listed):
                problems.append((path, line, literal, "走査に出るのに挙げていない", missing))
            for extra in sorted(listed - found):
                problems.append((path, line, literal, "挙げているのに走査に出ない", extra))
    return problems


def report_scans(directory):
    """`SCAN` の食い違いを印字し、件数を返す。"""
    rows = check_scans(directory)
    for path, line, literal, kind, (where, symbol) in rows:
        print(f"{os.path.relpath(path, REPO)}:{line}: SCAN `{literal}`: {kind} -- {where}: {symbol}")
    return rows


def rule_span(lines, symbol):
    """The half-open line range of a pest rule's definition, or `None` where it is not found.

    A grammar names its rules `name = { ... }`, so a rule is found by its name at the head of a
    line and ends where the braces the definition opens close again."""
    starts = re.compile(r"^\s*" + re.escape(symbol) + r"\s*=")
    for index, line in enumerate(lines):
        if starts.match(line):
            return item_body(lines, index)
    return None


def item_span(lines, symbol):
    """The half-open line range of a Rust item's definition, or `None` where it is not found.

    A one-part symbol names an item at any depth; a two-part `Owner::name` names one whose enclosing
    `impl`, `trait` or `mod` line carries the owner. An owner's block ends where its braces close,
    so a symbol names an item only while the scan is inside that block. The end is found by counting
    braces from the first one on or after the definition line, so an item without a body ends at its
    own line.
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
    depth, owner_exit, owner_open = 0, 0, False
    for index, line in enumerate(lines):
        if owner and scope.search(line):
            enclosing, owner_exit, owner_open = True, depth, False
        if starts.match(line) and enclosing:
            return item_body(lines, index)
        depth += line.count("{") - line.count("}")
        if owner and enclosing:
            owner_open = owner_open or depth > owner_exit
            if owner_open and depth <= owner_exit:
                enclosing = False
    return None


def item_body(lines, index):
    """The half-open line range of the item whose definition starts at `index`."""
    depth, seen = 0, False
    for end in range(index, len(lines)):
        depth += lines[end].count("{") - lines[end].count("}")
        seen = seen or "{" in lines[end]
        if seen and depth <= 0:
            return index, end + 1
        if not seen and lines[end].rstrip().endswith(";"):
            return index, end + 1
    return index, len(lines)


def digest(lines, span):
    """The digest of an item's source text, with the `// PROOF:` lines inside it left out.

    An item's annotation, and the annotations of the items nested in it, move whenever the
    propositions do. Reading them into the digest would report the item as changed every time the
    proof's own bookkeeping changes, which is the one thing the digest must not do."""
    body = "".join(l for l in lines[span[0]:span[1]] if not PROOF_COMMENT.match(l))
    return hashlib.sha256(body.encode("utf-8")).hexdigest()[:12]


def proposition_order(name):
    """Sort key: the frame first, then `P<n>` and its lettered variants, then everything else.

    The key orders the names totally. A set of them iterates in an order that varies from run to
    run, so a key that leaves two names tied would write the comment one way and then the other,
    and the comments would never settle."""
    if name == FRAME:
        return (-1, 0, "")
    numbered = re.fullmatch(r"P(\d+)([a-z]?)", name)
    if numbered:
        return (0, int(numbered.group(1)), numbered.group(2))
    return (1, 0, name)


def comment_for(props, directory):
    ordered = sorted(props, key=proposition_order)
    return f"// PROOF: {', '.join(ordered)} ({os.path.relpath(directory, REPO)})\n"


def strip_comments():
    """Take every `// PROOF:` line out of the sources, so the comments are rebuilt from nothing.

    A citation the proof stops making leaves its comment behind, and rebuilding around the leftovers
    would keep it. Stripping first also keeps the line positions the spans are computed at true."""
    for source in rust_files_with_comments():
        path = os.path.join(REPO, source)
        lines = open(path, encoding="utf-8").readlines()
        open(path, "w", encoding="utf-8").writelines(
            line for line in lines if not PROOF_COMMENT.match(line)
        )


def main():
    write = "--write" in sys.argv[1:]
    if write:
        strip_comments()
    findings = []
    for directory in proof_dirs():
        findings += report_scans(directory)
        cited = citations_of(directory)
        rows, edits = [], {}
        for (source, symbol), props in sorted(cited.items()):
            path = os.path.join(REPO, source)
            if not os.path.exists(path):
                findings.append(f"{source}: cited by {sorted(props)} and does not exist")
                continue
            if not source.endswith((".rs", ".pest")):
                rows.append((source, symbol, props, "-"))
                continue
            lines = open(path, encoding="utf-8").readlines()
            span = rule_span(lines, symbol) if source.endswith(".pest") else item_span(lines, symbol)
            if span is None:
                findings.append(f"{source}: `{symbol}` is cited and not found")
                continue
            rows.append((source, symbol, props, digest(lines, span)))
            if source.endswith(".pest"):
                continue
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
            ordered = ",".join(sorted(props, key=proposition_order))
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
            names = ",".join(sorted(props, key=proposition_order))
            yield f"{source}: `{symbol}` changed since the proof was written; re-verify {names}"


if __name__ == "__main__":
    sys.exit(main())
