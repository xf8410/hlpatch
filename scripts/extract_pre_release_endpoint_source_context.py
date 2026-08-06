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
sections = [
    "# Pre-release endpoint focused source context",
    "",
    f"source_commit: `{source_commit}`",
    "",
    "## Route and advertised-endpoint lines",
    "",
    "```rust",
    *route_lines,
    "```",
]
for start, name in starts:
    sections.extend([
        "",
        f"## `{name}` (starts at line {start + 1})",
        "",
        "```rust",
        *complete_function(start),
        "```",
    ])

output = Path("docs/PRE_RELEASE_ENDPOINT_SOURCE_AUDIT_CONTEXT.md")
output.write_text("\n".join(sections) + "\n", encoding="utf-8")
print(f"functions={len(starts)} route_lines={len(route_lines)} bytes={output.stat().st_size}")
