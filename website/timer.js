export const PHASES = {
  focus: { seconds: 25 * 60, name: "Focus", start: "Start a little focus" },
  short: { seconds: 5 * 60, name: "Short break", start: "Take a short break" },
  long: { seconds: 15 * 60, name: "Long break", start: "Take a longer break" },
};

export function formatTime(seconds) {
  const safe = Math.max(0, Math.ceil(seconds));
  return String(Math.floor(safe / 60)).padStart(2, "0") + ":" + String(safe % 60).padStart(2, "0");
}

export class DemoTimer {
  constructor(now = () => performance.now()) {
    this.now = now;
    this.phase = "focus";
    this.remaining = PHASES.focus.seconds;
    this.deadline = null;
  }
  get running() { return this.deadline !== null; }
  select(phase) {
    if (!Object.hasOwn(PHASES, phase)) throw new Error("Unknown timer phase");
    this.phase = phase;
    this.reset();
  }
  reset() {
    this.deadline = null;
    this.remaining = PHASES[this.phase].seconds;
  }
  start() {
    if (this.running) return;
    if (this.remaining <= 0) this.reset();
    this.deadline = this.now() + this.remaining * 1000;
  }
  pause() {
    this.tick();
    this.deadline = null;
  }
  tick() {
    if (this.running) {
      this.remaining = Math.max(0, (this.deadline - this.now()) / 1000);
      if (this.remaining === 0) this.deadline = null;
    }
    return this.remaining;
  }
}
