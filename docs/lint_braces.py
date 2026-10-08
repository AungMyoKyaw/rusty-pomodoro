#!/usr/bin/env python3
"""Report the first point in docs/parts/*.tex where the brace depth goes
negative, or where it is non-zero at end of file.  Ignores LaTeX comments
and escaped braces, which is exactly what TeX does."""
import glob
import sys


def scan(path):
    src = open(path).read()
    depth = 0
    line = 1
    i = 0
    while i < len(src):
        c = src[i]
        if c == "\n":
            line += 1
            i += 1
            continue
        if c == "%":
            while i < len(src) and src[i] != "\n":
                i += 1
            continue
        if c == "\\":
            i += 2
            continue
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth < 0:
                return line, "extra }"
        i += 1
    if depth != 0:
        return line, "unbalanced: depth %d at EOF" % depth
    return None


bad = 0
for path in sorted(glob.glob("parts/*.tex")) + ["architecture.tex"]:
    r = scan(path)
    if r:
        bad += 1
        print("%s:%d  %s" % (path, r[0], r[1]))
print("brace lint: %d problem(s)" % bad)
sys.exit(1 if bad else 0)