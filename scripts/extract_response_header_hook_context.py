from pathlib import Path

source = Path("hachimi_ura_plugin/src/lib.rs").read_text(encoding="utf-8")
needles = [
    "unity_send_hook_handler",
    "decompress_response_hook_handler",
    "install_api_sniff_hooks",
    "UNITY_SEND_ADDR",
    "push_sniff_metadata(",
    "SNIFF_RESPONSE_QUEUE",
]
sections = ["# Response header hook source contexts", ""]
for needle in needles:
    positions = []
    start = 0
    while True:
        index = source.find(needle, start)
        if index < 0:
            break
        positions.append(index)
        start = index + len(needle)
    sections.extend([f"## `{needle}` ({len(positions)} matches)", ""])
    for number, index in enumerate(positions, 1):
        left = max(0, index - 2500)
        right = min(len(source), index + 5000)
        sections.extend([f"### match {number}", "", "```rust", source[left:right], "```", ""])
Path("docs/RESPONSE_HEADER_HOOK_SOURCE_CONTEXT.md").write_text("\n".join(sections), encoding="utf-8")
