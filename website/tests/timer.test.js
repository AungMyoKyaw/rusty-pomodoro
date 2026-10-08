import { describe, expect, test } from "bun:test";
import { DemoTimer, PHASES, formatTime } from "../timer.js";

describe("browser timer", () => {
  test("formats and clamps remaining time", () => {
    expect(formatTime(1500)).toBe("25:00");
    expect(formatTime(299.1)).toBe("05:00");
    expect(formatTime(-4)).toBe("00:00");
    expect(formatTime(0)).toBe("00:00");
  });
  test("counts elapsed time, not interval callbacks", () => {
    let now = 100;
    const timer = new DemoTimer(() => now);
    timer.start();
    now += 123400;
    expect(timer.tick()).toBeCloseTo(1376.6);
    timer.pause();
    now += 500000;
    expect(timer.tick()).toBeCloseTo(1376.6);
    timer.start();
    now += 376600;
    expect(timer.tick()).toBeCloseTo(1000);
  });
  test("phase changes stop and reset running sessions", () => {
    const timer = new DemoTimer(() => 0);
    timer.start();
    timer.select("short");
    expect(timer.running).toBe(false);
    expect(timer.remaining).toBe(300);
    timer.select("long");
    expect(timer.remaining).toBe(900);
    expect(() => timer.select("__proto__")).toThrow("Unknown timer phase");
    expect(() => timer.select("unknown")).toThrow("Unknown timer phase");
  });
  test("completion stops exactly once and can restart", () => {
    let now = 0;
    const timer = new DemoTimer(() => now);
    timer.start();
    timer.start();
    now = PHASES.focus.seconds * 1000 + 1;
    expect(timer.tick()).toBe(0);
    expect(timer.running).toBe(false);
    timer.start();
    expect(timer.remaining).toBe(1500);
    expect(timer.running).toBe(true);
    timer.reset();
    expect(timer.running).toBe(false);
    expect(timer.remaining).toBe(1500);
  });
});
