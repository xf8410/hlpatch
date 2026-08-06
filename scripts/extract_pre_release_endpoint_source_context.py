from pathlib import Path
import re
import subprocess

source_path = Path("hachimi_ura_plugin/src/lib.rs")
text = source_path.read_text(encoding="utf-8")
lines = text.splitlines()
terms = (
    "summary", "health", "inherit", "compat", "saddle", "hall",
    "storage_", "method_index", "method_detail", "method_by_addr",
    "nested_types", "enum_values", "parse_query", "query_pair",
    "not_found", "available",
)


def complete_function(start: int) -> list[str]:
    depth = 0
    opened = False
    output = []
    for line in lines[start:]:
        output.append(line)
        code = re.sub(r'"(?:\\.|[^"\\])*"', '""', line)
        for char in code:
            if char == "{":
                depth += 1
                opened = True
            elif char == "}":
                depth -= 1
        if opened and depth == 0:
            return output
    raise RuntimeError(f"unterminated function at line {start + 1}")


starts = []
signature = re.compile(r"^\s*(?:pub\s+)?(?:unsafe\s+)?fn\s+([A-Za-z0-9_]+)")
for index, line in enumerate(lines):
    match = signature.match(line)
    if match and any(term in match.group(1).lower() for term in terms):
        starts.append((index, match.group(1)))

route_lines = []
for index, line in enumerate(lines, 1):
    if re.search(r"path\s*(?:==|\.starts_with\()", line) or '"available"' in line or "endpoints" in line:
        route_lines.append(f"{index}: {line}")

source_commit = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
out_dir = Path("docs/pre_release_endpoint_source")
out_dir.mkdir(parents=True, exist_ok=True)

index_lines = [
    "# Pre-release endpoint focused source index",
    "",
    f"source_commit: `{source_commit}`",
    "",
    "| function | source line | file |",
    "|---|---:|---|",
]

for start, name in starts:
    body = complete_function(start)
    safe_name = re.sub(r"[^A-Za-z0-9_.-]", "_", name)
    output = out_dir / f"{start + 1:06d}_{safe_name}.md"
    output.write_text(
        "\n".join([
            f"# `{name}`",
            "",
            f"source_commit: `{source_commit}`",
            f"source_line: `{start + 1}`",
            "",
            "```rust",
            *body,
            "```",
            "",
        ]),
        encoding="utf-8",
    )
    index_lines.append(f"| `{name}` | {start + 1} | `{output.name}` |")

routes_output = out_dir / "routes_and_advertised_endpoints.md"
routes_output.write_text(
    "\n".join([
        "# Routes and advertised endpoints",
        "",
        f"source_commit: `{source_commit}`",
        "",
        "```rust",
        *route_lines,
        "```",
        "",
    ]),
    encoding="utf-8",
)
index_lines.extend(["", f"Route lines: `{routes_output.name}`", ""])
(out_dir / "README.md").write_text("\n".join(index_lines), encoding="utf-8")

# Keep one aggregate artifact for local/Actions inspection, but bounded files are the review source.
aggregate = [
    "# Pre-release endpoint focused source context",
    "",
    f"source_commit: `{source_commit}`",
    "",
    "See `docs/pre_release_endpoint_source/README.md` for bounded per-function files.",
    "",
]
Path("docs/PRE_RELEASE_ENDPOINT_SOURCE_AUDIT_CONTEXT.md").write_text(
    "\n".join(aggregate), encoding="utf-8"
)
print(f"functions={len(starts)} route_lines={len(route_lines)}")
