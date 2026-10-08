import { expect, test } from "bun:test";
import { readFileSync, existsSync } from "node:fs";
import { resolve } from "node:path";
const root = resolve(import.meta.dir, "..");
const html = readFileSync(resolve(root, "index.html"), "utf8");
const css = readFileSync(resolve(root, "styles.css"), "utf8");

test("all local HTML, CSS and screenshot assets exist", () => {
  const refs = [...html.matchAll(/(?:src|href)="([^"]+)"/g)].map((match) => match[1]);
  for (const ref of refs.filter((ref) => !/^(https?:|#|\.\/)/.test(ref))) {
    expect(existsSync(resolve(root, ref))).toBe(true);
  }
  for (const match of css.matchAll(/url\("([^"]+)"\)/g)) {
    expect(existsSync(resolve(root, match[1]))).toBe(true);
  }
  for (const name of ["timer", "settings", "statistics"]) {
    expect(existsSync(resolve(root, "assets", name + ".webp"))).toBe(true);
  }
});
test("local navigation targets are real and IDs are unique", () => {
  const ids = [...html.matchAll(/\bid="([^"]+)"/g)].map((match) => match[1]);
  expect(new Set(ids).size).toBe(ids.length);
  for (const match of html.matchAll(/href="#([^"]+)"/g)) expect(ids).toContain(match[1]);
});
test("ships semantic content, truthful support and progressive enhancement", () => {
  expect(html).toContain('lang="en"');
  expect([...html.matchAll(/<h1\b/g)].length).toBe(1);
  expect(html).toContain("<noscript>");
  expect(html).toContain("runtime is not yet tested");
  expect(html).toContain("not notarized");
  expect(html).toContain('aria-live="polite"');
  expect(css).toContain("prefers-reduced-motion:reduce");
  expect(css).toContain(":focus-visible");
  expect(css).not.toContain("@import");
  for (const match of html.matchAll(/<img\b([^>]+)>/g)) {
    expect(match[1]).toMatch(/\balt="/);
    expect(match[1]).toMatch(/\bwidth="/);
    expect(match[1]).toMatch(/\bheight="/);
  }
});
test("body copy and actions have accessible palette contrast", () => {
  function luminance(hex) {
    const channels = hex.match(/[0-9a-f]{2}/g).map((value) => parseInt(value, 16) / 255);
    return channels.map((v) => v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4)
      .reduce((sum, v, i) => sum + v * [.2126, .7152, .0722][i], 0);
  }
  for (const [foreground, background] of [["616359", "f6f3eb"], ["f6f3eb", "bd3c2c"], ["d1d8c9", "233c32"], ["616359", "ede7da"]]) {
    const values = [luminance(foreground), luminance(background)].sort((a, b) => b - a);
    expect((values[0] + .05) / (values[1] + .05)).toBeGreaterThanOrEqual(4.5);
  }
});
