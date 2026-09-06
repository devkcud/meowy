import argparse
import html
import os
import re
import unicodedata
from pathlib import Path
from urllib.parse import unquote, urlsplit


ROOT = Path(__file__).resolve().parents[1]
LINK = re.compile(
    r"\[(?:\\.|[^\[\]\\]|\[[^\[\]]*\])*\]\(\s*"
    r"(?:<(?P<angle>[^>\n]*)>|(?P<plain>(?:\\.|[^\s()\\]|\([^()\s]*\))*))"
    r"(?:\s+(?:\"[^\"]*\"|'[^']*'|\([^)]*\)))?\s*\)"
)


def prose(text):
    lines = []
    fence = None
    for line in text.splitlines(keepends=True):
        mark = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", line)
        if fence:
            if mark and mark[1][0] == fence[0] and len(mark[1]) >= len(fence) and not mark[2].strip():
                fence = None
            lines.append(re.sub(r"[^\n]", " ", line))
        elif mark:
            fence = mark[1]
            lines.append(re.sub(r"[^\n]", " ", line))
        else:
            lines.append(line)
    return re.sub(r"<!--.*?-->", lambda match: re.sub(r"[^\n]", " ", match[0]), "".join(lines), flags=re.DOTALL)


def links(text):
    text = prose(text)
    text = re.sub(r"(`+)(?!`)(.*?)(?<!`)\1(?!`)", lambda match: re.sub(r"[^\n]", " ", match[0]), text, flags=re.DOTALL)
    for match in LINK.finditer(text):
        target = match["angle"] if match["angle"] is not None else match["plain"]
        yield text.count("\n", 0, match.start()) + 1, re.sub(r"\\(.)", r"\1", target)


def anchors(text):
    seen = set()
    for match in re.finditer(r"^ {0,3}#{1,6}[ \t]+(.+?)[ \t]*$", prose(text), re.MULTILINE):
        title = re.sub(r"[ \t]+#+[ \t]*$", "", match[1])
        title = LINK.sub(lambda link: link[0][1:link[0].index("](")], title)
        title = html.unescape(re.sub(r"<[^>]*>", "", title)).lower()
        base = "".join(char for char in title if char in " -_" or unicodedata.category(char)[0] in "LNM").replace(" ", "-")
        name = base
        count = 0
        while name in seen:
            count += 1
            name = f"{base}-{count}"
        seen.add(name)
    return seen


def documents(root):
    files = list(root.glob("*.md"))
    for name in ("docs", "editor", "tools", "compiler", "runtime"):
        for base, dirs, names in os.walk(root / name):
            dirs[:] = sorted(name for name in dirs if name not in {"target", "build", "__pycache__"})
            files.extend(Path(base) / name for name in names if name.endswith(".md"))
    return sorted(files)


def check(root):
    root = root.resolve()
    if not root.is_dir():
        raise ValueError(f"repository root is not a directory: {root}")
    files = documents(root)
    errors = []
    count = 0
    cache = {}
    for source in files:
        for line, target in links(source.read_text(encoding="utf-8")):
            url = urlsplit(target)
            if url.scheme or url.netloc:
                continue
            count += 1
            path = unquote(url.path)
            dest = (root / path.lstrip("/") if path.startswith("/") else source.parent / path).resolve() if path else source.resolve()
            label = f"{source.relative_to(root)}:{line}: {target!r}"
            if not dest.is_relative_to(root):
                errors.append(f"{label}: target escapes repository")
            elif not dest.is_file():
                errors.append(f"{label}: target is not a file")
            elif url.fragment and dest.suffix == ".md":
                if dest not in cache:
                    cache[dest] = anchors(dest.read_text(encoding="utf-8"))
                if unquote(url.fragment) not in cache[dest]:
                    errors.append(f"{label}: heading does not exist")
    return len(files), count, errors


def main():
    parser = argparse.ArgumentParser(description="Check local inline Markdown links and ATX heading anchors; no network requests.")
    parser.add_argument("--root", type=Path, default=ROOT)
    args = parser.parse_args()
    try:
        files, count, errors = check(args.root)
    except (OSError, UnicodeError, ValueError) as error:
        print(f"FAIL {error}")
        return 1
    for error in errors:
        print(f"FAIL {error}")
    print(f"Checked {count} local links in {files} Markdown files; {len(errors)} failures. External links were not fetched.")
    return int(bool(errors))


if __name__ == "__main__":
    raise SystemExit(main())
