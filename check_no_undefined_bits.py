"""Report the values carrying undefined bits that reach a function boundary.

LLVM lets a caller state that an argument or a result holds no undefined bit (`noundef`), and reads a
violation of that statement as undefined behavior. The statement is therefore worth only as much as
the guarantee behind it, and the guarantee here is that the code generator writes every bit of every
value it hands across a boundary.

This reads the LLVM IR the code generator emits, before any LLVM pass runs, and answers whether that
holds: it follows each `poison` and `undef` the module names, through the instructions that carry it,
and reports every one that arrives at a call argument or a `ret`.

    python3 check_no_undefined_bits.py <module.ll> [<module.ll> ...]

Exits 0 when nothing arrives, 1 when something does, and 2 when a file cannot be read.

**Examples**

    %0 = insertvalue { i64, i64 } poison, i64 %a, 0
    %1 = insertvalue { i64, i64 } %0, i64 %b, 1
    call void @f({ i64, i64 } %1)            ->  nothing: both fields were written
    call void @f({ i64, i64 } %0)            ->  reported: field 1 is still poison

The walk is per field rather than per byte: a scaffold built by `insertvalue` from `poison` is clean
once every field has been written, which is how the code generator builds an aggregate. `freeze`
makes its result clean, which is what that instruction is for.

Two things it does not follow. A value stored to memory and loaded back is clean here, so a field
left unwritten in an allocated object is invisible to this walk -- what the walk covers is the value
a function hands to another function. And a value a caller passes in is clean, since this asks what
this module hands over rather than what it receives.
"""

import re
import sys
from typing import Dict, List, Optional, Tuple

LOCAL = re.compile(r"%(?:[-a-zA-Z$._0-9]+|\"[^\"]*\")")
UNDEFINED_CONSTANT = re.compile(r"\b(?:poison|undef)\b")
ASSIGNMENT = re.compile(r"^\s*(%(?:[-a-zA-Z$._0-9]+|\"[^\"]*\"))\s*=\s*(.*)$")
FUNCTION_HEADER = re.compile(r"^define\b.*?@([^(]*)\(")
ARRAY_TYPE = re.compile(r"^\[\s*(\d+)\s*x\s")


class Taint:
    """Which of a value's bits are undefined, as a tree over the value's fields.

    A leaf is `ALL` or `NONE`, covering the whole value. A node carries one child per field, which is
    what lets an aggregate written field by field start undefined and end defined: the node is clean
    once every child is.
    """

    __slots__ = ("fields", "undefined")

    def __init__(self, undefined: bool, fields: Optional[Dict[int, "Taint"]] = None):
        self.undefined = undefined
        self.fields = fields

    def is_clean(self) -> bool:
        if self.fields is None:
            return not self.undefined
        return all(field.is_clean() for field in self.fields.values())

    def spread(self, arity: int) -> "Taint":
        """This taint as a node of `arity` fields, so that one of them can be written."""
        if self.fields is not None:
            return self
        return Taint(self.undefined, {i: self for i in range(arity)})

    def field(self, idx: int) -> "Taint":
        if self.fields is None:
            return self
        return self.fields.get(idx, self)

    def at(self, path: Tuple[int, ...]) -> "Taint":
        node = self
        for idx in path:
            node = node.field(idx)
        return node

    def written_at(self, idx: int, sub: "Taint", arity: int) -> "Taint":
        node = self.spread(arity)
        fields = dict(node.fields)
        fields[idx] = sub
        return Taint(node.undefined, fields)

    def union(self, other: "Taint") -> "Taint":
        if self.fields is None and other.fields is None:
            return ALL if (self.undefined or other.undefined) else NONE
        if self.fields is None:
            return ALL if self.undefined else other
        if other.fields is None:
            return ALL if other.undefined else self
        keys = set(self.fields) | set(other.fields)
        return Taint(False, {i: self.field(i).union(other.field(i)) for i in keys})


ALL = Taint(True)
NONE = Taint(False)


def split_operands(text: str) -> List[str]:
    """`text` cut at the commas between operands.

    A type is written with commas inside it -- `{ ptr, i64 }`, `[2 x i64]`, `<4 x i64>` -- so the cut
    is made only where no bracket is open.
    """
    depth = 0
    operands: List[str] = []
    current = ""
    for character in text:
        if character in "{[(<":
            depth += 1
        elif character in "}])>":
            depth -= 1
        if character == "," and depth == 0:
            operands.append(current)
            current = ""
            continue
        current += character
    operands.append(current)
    return [operand.strip() for operand in operands if operand.strip()]


def operand_type(operand: str) -> str:
    """The type an operand is written with: everything before the value it names.

    **Examples**

        "{ i64, i64 } %0"  ->  "{ i64, i64 }"
        "i64 %a"           ->  "i64"
    """
    return operand.rsplit(" ", 1)[0].strip() if " " in operand else ""


def arity_of(type_text: str) -> int:
    """How many fields a value of `type_text` holds, and 0 where it holds no field."""
    if type_text.startswith("{") and type_text.endswith("}"):
        return len(split_operands(type_text[1:-1]))
    array = ARRAY_TYPE.match(type_text)
    if array:
        return int(array.group(1))
    return 0


def taint_of(operand: str, taints: Dict[str, Taint]) -> Taint:
    """The undefined bits of one operand, whether it names a value or a constant."""
    if UNDEFINED_CONSTANT.search(operand):
        return ALL
    names = LOCAL.findall(operand)
    return taints.get(names[0], NONE) if names else NONE


def field_indices(operands: List[str]) -> Tuple[int, ...]:
    """The constant field indices an `insertvalue` or `extractvalue` ends with."""
    return tuple(int(operand) for operand in operands if operand.isdigit())


def taint_of_result(expression: str, taints: Dict[str, Taint]) -> Taint:
    """The undefined bits of the value an instruction that reads its operands whole produces."""
    if UNDEFINED_CONSTANT.search(expression):
        return ALL
    for name in LOCAL.findall(expression):
        if not taints.get(name, NONE).is_clean():
            return ALL
    return NONE


def check_function(name: str, body: List[str], path: str) -> List[str]:
    """The lines of `body` that hand a value with undefined bits to a caller or a callee."""
    taints: Dict[str, Taint] = {}
    findings: List[str] = []

    for line in body:
        text = line.strip()
        if not text or text.startswith(";"):
            continue

        assignment = ASSIGNMENT.match(line)
        expression = assignment.group(2) if assignment else text
        words = expression.split()
        opcode = words[0] if words else ""

        # What the instruction hands over, taken before what it defines: a call that returns a value
        # does both.
        if "call" in words[:3] or opcode == "invoke":
            arguments = (
                expression[expression.find("(") + 1 : expression.rfind(")")] if "(" in expression else ""
            )
            for operand in split_operands(arguments):
                if not taint_of(operand, taints).is_clean():
                    findings.append(f"{path}: {name}: argument `{operand}`: {text}")
        elif opcode == "ret" and len(words) > 1:
            for operand in split_operands(expression[len("ret") :]):
                if not taint_of(operand, taints).is_clean():
                    findings.append(f"{path}: {name}: result `{operand}`: {text}")

        if not assignment:
            continue
        defined = assignment.group(1)

        if opcode == "freeze":
            taints[defined] = NONE
        elif opcode == "insertvalue":
            operands = split_operands(expression[len("insertvalue") :])
            aggregate = taint_of(operands[0], taints)
            element = taint_of(operands[1], taints) if len(operands) > 1 else NONE
            indices = field_indices(operands[2:])
            arity = arity_of(operand_type(operands[0]))
            # The code generator writes one field at a time; a deeper path is read whole.
            taints[defined] = (
                aggregate.written_at(indices[0], element, arity) if len(indices) == 1 else ALL
            )
        elif opcode == "extractvalue":
            operands = split_operands(expression[len("extractvalue") :])
            taints[defined] = taint_of(operands[0], taints).at(field_indices(operands[1:]))
        elif opcode == "phi":
            taint = NONE
            for incoming in re.findall(r"\[([^\]]*)\]", expression):
                taint = taint.union(taint_of(incoming.split(",")[0], taints))
            taints[defined] = taint
        elif opcode in ("load", "alloca"):
            # Memory is outside this walk; see the module docstring.
            taints[defined] = NONE
        else:
            taints[defined] = taint_of_result(expression, taints)

    return findings


def check_module(path: str) -> List[str]:
    with open(path, "r", errors="replace") as handle:
        lines = handle.read().splitlines()

    findings: List[str] = []
    name: Optional[str] = None
    body: List[str] = []
    for line in lines:
        header = FUNCTION_HEADER.match(line)
        if header:
            name = header.group(1)
            body = []
            continue
        if name is None:
            continue
        if line.startswith("}"):
            findings.extend(check_function(name, body, path))
            name = None
            continue
        body.append(line)
    return findings


def main(argv: List[str]) -> int:
    if len(argv) < 2:
        print(__doc__)
        return 2
    findings: List[str] = []
    for path in argv[1:]:
        try:
            findings.extend(check_module(path))
        except OSError as error:
            print(f"cannot read {path}: {error}", file=sys.stderr)
            return 2
    for finding in findings:
        print(finding)
    print(f"{len(findings)} values with undefined bits reach a boundary, over {len(argv) - 1} modules")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
