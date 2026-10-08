#!/usr/bin/env python3
"""Scan docs/parts/*.tex for TikZ \\node text arguments that are not closed
before the node's terminating semicolon.  Catches the most common LaTeX
mistake made while hand-writing diagrams: a node body like

    \\node[b] (a) at (0,0) {\\textbf{x}}\\\\[1mm]{\\tiny y};

which closes the inner group but not the node."""
import glob
import re
import sys


def check(path):
    src = open(path).read()
    problems = []
    for m in re.finditer(r"\\node\b", src):
        i = m.end()
        sq = par = 0
        while i < len(src):
            c = src[i]
            if c == "[":
                sq += 1
            elif c == "]":
                sq -= 1
            elif c == "(":
                par += 1
            elif c == ")":
                par -= 1
            elif c == "{" and sq == 0 and par == 0:
                break
            elif c == ";" and sq == 0 and par == 0:
                break
            i += 1
        if i >= len(src) or src[i] != "{":
            continue
        start = i
        d = 0
        while i < len(src):
            if src[i] == "\\":
                i += 2
                continue
            if src[i] == "{":
                d += 1
            elif src[i] == "}":
                d -= 1
                if d == 0:
                    break
            i += 1
        if i >= len(src):
            continue
        nxt = re.match(r"\s*(\\|\{|;|\))", src[i + 1:])
        if not nxt or nxt.group(1) != ";":
            line = src.count("\n", 0, start) + 1
            problems.append((line, " ".join(src[start:start + 72].split())))
    return problems


bad = 0
for path in sorted(glob.glob("parts/*.tex")):
    for line, snippet in check(path):
        bad += 1
        print(f"{path}:{line}  {snippet}")
print("node-argument lint: %d problem(s)" % bad)
sys.exit(1 if bad else 0)