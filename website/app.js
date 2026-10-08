import { DemoTimer, PHASES, formatTime } from "./timer.js";

const screens = {
  timer: { src: "assets/timer.webp", alt: "Timer screen with a 25-minute focus session and start, restart, skip, and stop controls", caption: "Focus and break controls in one place." },
  settings: { src: "assets/settings.webp", alt: "Rusty Pomodoro settings for focus duration, short and long breaks, and sessions per cycle", caption: "Your durations. Your cycle. Your way of working." },
  statistics: { src: "assets/statistics.webp", alt: "Rusty Pomodoro statistics with today's totals, an empty seven-day history, and CSV export", caption: "Today and the last seven days, with CSV export. Screenshot shows a fresh history." },
};
const image = document.querySelector("#gallery-image");
const caption = document.querySelector("#screen-caption");
let imageRequest = 0;
const picker = document.querySelector(".screen-picker");
picker.hidden = false;
picker.addEventListener("click", async (event) => {
  const button = event.target.closest("[data-screen]");
  if (!button) return;
  const screen = screens[button.dataset.screen];
  const request = ++imageRequest;
  caption.textContent = "Loading screenshot…";
  const preload = new Image();
  preload.src = screen.src;
  try {
    await preload.decode();
    if (request !== imageRequest) return;
    image.src = screen.src;
    image.alt = screen.alt;
    caption.textContent = screen.caption;
    picker.querySelectorAll("button").forEach((item) => item.setAttribute("aria-pressed", String(item === button)));
  } catch {
    if (request === imageRequest) caption.textContent = "Screenshot could not load. Try another view or reload this page.";
  }
});

const timer = new DemoTimer();
const display = document.querySelector("#demo-time");
const status = document.querySelector("#timer-status");
const toggle = document.querySelector("#toggle-timer");
const phases = document.querySelector(".phase-picker");
let interval = null;

function paint() {
  const seconds = Math.ceil(timer.tick());
  display.textContent = formatTime(seconds);
  display.setAttribute("aria-label", seconds + " seconds remaining");
  toggle.textContent = timer.running ? "Pause" : timer.remaining <= 0 ? "Start again" : PHASES[timer.phase].start;
}
function stopInterval() {
  if (interval !== null) clearInterval(interval);
  interval = null;
}
function reset() {
  stopInterval();
  timer.reset();
  status.textContent = PHASES[timer.phase].name + " ready. Take it at your own pace.";
  paint();
}
toggle.addEventListener("click", () => {
  if (timer.running) {
    timer.pause();
    stopInterval();
    status.textContent = "Paused. Your time is still yours.";
  } else {
    timer.start();
    status.textContent = PHASES[timer.phase].name + " in progress.";
    interval = setInterval(() => {
      paint();
      if (!timer.running) {
        stopInterval();
        status.textContent = "Session complete. Take a breath. Choose what comes next.";
      }
    }, 250);
  }
  paint();
});
document.querySelector("#reset-timer").addEventListener("click", reset);
phases.addEventListener("click", (event) => {
  const button = event.target.closest("[data-phase]");
  if (!button) return;
  timer.select(button.dataset.phase);
  phases.querySelectorAll("button").forEach((item) => item.setAttribute("aria-pressed", String(item === button)));
  reset();
});
document.addEventListener("visibilitychange", () => {
  paint();
  if (timer.remaining <= 0) {
    stopInterval();
    status.textContent = "Session complete. Take a breath. Choose what comes next.";
  }
});
phases.hidden = false;
document.querySelector(".demo-controls").hidden = false;
