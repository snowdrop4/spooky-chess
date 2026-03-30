from __future__ import annotations

import ast
from collections.abc import Iterable
import inspect
from pathlib import Path


def is_string_expr(node: ast.stmt) -> bool:
    return isinstance(node, ast.Expr) and isinstance(node.value, ast.Constant) and isinstance(node.value.value, str)


def iter_documented_members(body: list[ast.stmt]) -> Iterable[tuple[ast.stmt, str | None]]:
    index = 0
    while index < len(body):
        node = body[index]
        if is_string_expr(node):
            index += 1
            continue

        docstring: str | None = None
        if isinstance(node, ast.AnnAssign):
            next_index = index + 1
            if next_index < len(body) and is_string_expr(body[next_index]):
                value = body[next_index]
                assert isinstance(value, ast.Expr)
                assert isinstance(value.value, ast.Constant)
                assert isinstance(value.value.value, str)
                docstring = value.value.value
                index = next_index
        elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            docstring = ast.get_docstring(node)

        yield node, docstring
        index += 1


def format_annotation(node: ast.AST | None) -> str | None:
    if node is None:
        return None
    return ast.unparse(node)


def format_argument(arg: ast.arg, default: ast.AST | None = None) -> str:
    rendered = arg.arg
    annotation = format_annotation(arg.annotation)
    if annotation:
        rendered = f"{rendered}: {annotation}"
    if default is not None:
        rendered = f"{rendered} = {ast.unparse(default)}"
    return rendered


def format_function_signature(node: ast.FunctionDef | ast.AsyncFunctionDef) -> str:
    parts: list[str] = []
    positional = [*node.args.posonlyargs, *node.args.args]
    positional_defaults = [None] * (len(positional) - len(node.args.defaults)) + list(node.args.defaults)

    for index, (arg, default) in enumerate(zip(positional, positional_defaults, strict=False), start=1):
        parts.append(format_argument(arg, default))
        if node.args.posonlyargs and index == len(node.args.posonlyargs):
            parts.append("/")

    if node.args.vararg is not None:
        parts.append(f"*{format_argument(node.args.vararg)}")
    elif node.args.kwonlyargs:
        parts.append("*")

    for arg, default in zip(node.args.kwonlyargs, node.args.kw_defaults, strict=False):
        parts.append(format_argument(arg, default))

    if node.args.kwarg is not None:
        parts.append(f"**{format_argument(node.args.kwarg)}")

    signature = f"{node.name}({', '.join(parts)})"
    returns = format_annotation(node.returns)
    if returns:
        signature = f"{signature} -> {returns}"
    return signature


def format_docstring(docstring: str | None, indent: str = "") -> list[str]:
    if not docstring:
        return [f"{indent}Undocumented."]

    cleaned = inspect.cleandoc(docstring)
    if not cleaned:
        return [f"{indent}Undocumented."]

    lines = cleaned.splitlines()
    return [f"{indent}{line}" if line else "" for line in lines]


def render_data_directive(name: str, annotation: str | None, docstring: str | None, indent: str = "") -> list[str]:
    lines = [f"{indent}.. py:data:: {name}"]
    if annotation:
        lines.append(f"{indent}   :type: {annotation}")
    lines.append("")
    lines.extend(format_docstring(docstring, indent=f"{indent}   "))
    lines.append("")
    return lines


def render_attribute_directive(name: str, annotation: str | None, docstring: str | None, indent: str = "") -> list[str]:
    lines = [f"{indent}.. py:attribute:: {name}"]
    if annotation:
        lines.append(f"{indent}   :type: {annotation}")
    lines.append("")
    lines.extend(format_docstring(docstring, indent=f"{indent}   "))
    lines.append("")
    return lines


def render_function_directive(signature: str, docstring: str | None, indent: str = "") -> list[str]:
    lines = [f"{indent}.. py:function:: {signature}", ""]
    lines.extend(format_docstring(docstring, indent=f"{indent}   "))
    lines.append("")
    return lines


def render_method_directive(signature: str, docstring: str | None, indent: str = "") -> list[str]:
    lines = [f"{indent}.. py:method:: {signature}", ""]
    lines.extend(format_docstring(docstring, indent=f"{indent}   "))
    lines.append("")
    return lines


def should_render_member(name: str, docstring: str | None) -> bool:
    return bool(docstring) or not (name.startswith("__") and name.endswith("__"))


def render_class_directive(node: ast.ClassDef, docstring: str | None) -> list[str]:
    lines = [node.name, "-" * len(node.name), "", f".. py:class:: {node.name}", ""]
    lines.extend(format_docstring(docstring, indent="   "))
    lines.append("")

    attributes: list[list[str]] = []
    methods: list[list[str]] = []

    for member, member_docstring in iter_documented_members(node.body):
        if isinstance(member, ast.AnnAssign):
            name = ast.unparse(member.target)
            if should_render_member(name, member_docstring):
                attributes.append(
                    render_attribute_directive(
                        name, format_annotation(member.annotation), member_docstring, indent="   "
                    )
                )
            continue

        if not isinstance(member, (ast.FunctionDef, ast.AsyncFunctionDef)):
            continue

        if not should_render_member(member.name, member_docstring):
            continue

        is_property = any(
            isinstance(decorator, ast.Name) and decorator.id == "property" for decorator in member.decorator_list
        )
        if is_property:
            attributes.append(
                render_attribute_directive(
                    member.name, format_annotation(member.returns), member_docstring, indent="   "
                )
            )
            continue

        methods.append(render_method_directive(format_function_signature(member), member_docstring, indent="   "))

    if attributes:
        lines.extend(["   .. rubric:: Attributes", ""])
        for block in attributes:
            lines.extend(block)

    if methods:
        lines.extend(["   .. rubric:: Methods", ""])
        for block in methods:
            lines.extend(block)

    return lines


def generate_stub_reference(stub_path: Path, output_path: Path) -> None:
    source = stub_path.read_text(encoding="utf-8")
    module = ast.parse(source, filename=str(stub_path))

    constants: list[tuple[str, str | None, str | None]] = []
    functions: list[tuple[str, str | None]] = []
    classes: list[tuple[ast.ClassDef, str | None]] = []

    for node, docstring in iter_documented_members(module.body):
        if isinstance(node, ast.AnnAssign):
            constants.append((ast.unparse(node.target), format_annotation(node.annotation), docstring))
        elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            functions.append((format_function_signature(node), docstring))
        elif isinstance(node, ast.ClassDef):
            classes.append((node, docstring))

    lines = [
        ".. This file is generated by docs/_ext/generate_stub_reference.py.",
        "",
        ".. currentmodule:: spooky_chess",
        "",
        "The reference below is generated from the published typing stub.",
        "",
    ]

    if constants:
        lines.extend(["Constants", "---------", ""])
        for name, annotation, docstring in constants:
            lines.extend(render_data_directive(name, annotation, docstring))

    if functions:
        lines.extend(["Functions", "---------", ""])
        for signature, docstring in functions:
            lines.extend(render_function_directive(signature, docstring))

    if classes:
        lines.extend(["Classes", "-------", ""])
        for node, docstring in classes:
            lines.extend(render_class_directive(node, docstring))

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def main() -> None:
    root_dir = Path(__file__).resolve().parents[2]
    generate_stub_reference(
        root_dir / "spooky_chess.pyi", root_dir / "docs" / "_generated" / "python_api_reference.rst"
    )


if __name__ == "__main__":
    main()
