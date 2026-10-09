#!/usr/bin/env python3
"""Verify startup visibility and actual GUI foreground/Dock state of isolated apps."""
import json
import os
from pathlib import Path
import plistlib
import signal
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def app_pid(parent, executable=None):
    rows = subprocess.check_output(["ps", "-axo", "pid=,ppid=,comm="], text=True)
    processes = [row.strip().split(None, 2) for row in rows.splitlines()]
    if executable:
        return next((int(pid) for pid, _, command in processes
                     if Path(command).resolve() == executable.resolve()), None)
    descendants = {parent}
    while True:
        added = {int(pid) for pid, ppid, _ in processes if int(ppid) in descendants}
        if added <= descendants:
            break
        descendants |= added
    return next((int(pid) for pid, _, command in processes
                 if int(pid) in descendants and Path(command).name == "rusty-pomodoro"), None)


def observe(pid, observer, gui):
    windows = json.loads(subprocess.check_output([str(observer), str(pid)], text=True))
    state = {"pid": pid, "timerVisible": any(
        window.get("kCGWindowName") == "Rusty Pomodoro" for window in windows)}
    if gui:
        result = subprocess.run(
            ["osascript", "-e", 'tell application "System Events" to get frontmost of first '
             f'application process whose unix id is {pid}'],
            capture_output=True, text=True,
        )
        state["active"] = result.returncode == 0 and result.stdout.strip() == "true"
    return state


def main():
    if sys.platform != "darwin":
        sys.exit("Startup/Dock checks require macOS.")
    subprocess.run(["cargo", "build", "--locked", "--bin", "rusty-pomodoro"],
                   cwd=ROOT, check=True)
    with tempfile.TemporaryDirectory(prefix="rusty-startup-smoke-") as directory:
        work = Path(directory)
        observer = work / "window-state"
        subprocess.run(["xcrun", "swiftc", str(ROOT / "scripts/window-info-macos.swift"),
                        "-o", str(observer)], check=True)
        binary = str(ROOT / "target/debug/rusty-pomodoro")
        cases = [
            ("ordinary-visible", [binary], False, True, False, False),
            ("make-dev-visible", ["make", "dev"], False, True, False, True),
            ("make-dev-hidden-setting", ["make", "dev"], True, True, False, True),
            ("ordinary-hidden", [binary], True, False, False, False),
            ("explicit-show", [binary, "--show"], True, True, False, False),
            ("benchmark-visible", [binary], False, True, True, False),
        ]
        background = subprocess.check_output(["launchctl", "managername"], text=True).strip() != "Aqua"
        for name, command, hidden, visible, benchmark, gui in cases:
            if background and not gui:
                print(f"SKIP {name}: Background launches cannot assert the active desktop Space.", flush=True)
                continue
            config = work / name
            config.mkdir()
            settings = config / "settings.conf"
            original = f"hide_on_launch={str(hidden).lower()}\n"
            settings.write_text(original)
            env = {key: value for key, value in os.environ.items()
                   if not key.startswith(("RUSTY_POMODORO_", "TOMITO_"))}
            env["RUSTY_POMODORO_CONFIG_DIR"] = str(config)
            launch_tmp = config / "launch-tmp"
            launch_tmp.mkdir()
            env["TMPDIR"] = str(launch_tmp)
            if benchmark:
                env["RUSTY_POMODORO_BENCHMARK"] = "1"
            bundle = work / f"Rusty Startup Test {os.getpid()} {name}.app"
            if gui:
                env["RUSTY_POMODORO_DEV_BUNDLE"] = str(bundle)
            executable = bundle / "Contents/MacOS/rusty-pomodoro" if gui else None
            pid = None
            with (work / f"{name}.log").open("w+") as log:
                process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=log,
                                           stderr=log, start_new_session=True)
                try:
                    deadline = time.monotonic() + 20
                    state = None
                    while time.monotonic() < deadline:
                        pid = app_pid(process.pid, executable)
                        if pid:
                            state = observe(pid, observer, gui)
                            if state["timerVisible"] == visible and (not gui or state["active"]):
                                time.sleep(0.4)
                                stable = observe(pid, observer, gui)
                                # Observe startup focus once; do not require stealing it back
                                # if the user switches apps during the stability check.
                                if stable["timerVisible"] == visible:
                                    break
                        if process.poll() not in (None, 0):
                            log.seek(0)
                            raise RuntimeError(f"{name} exited: {log.read()}")
                        time.sleep(0.1)
                    else:
                        log.seek(0)
                        raise AssertionError(f"{name}: {state}; log: {log.read()}")
                    if gui:
                        assert process.poll() is None, "dev launcher returned while its app is alive"
                        log.flush()
                        log.seek(0)
                        launcher_output = log.read()
                        assert "GetProcessPID" not in launcher_output, "open -W waiting error regressed"
                        assert f"Running Slint dev app (PID {pid})" in launcher_output, launcher_output
                        subprocess.run(
                            ["osascript", "-e", 'tell application "System Events" to tell process '
                             f'"Dock" to get name of UI element "{bundle.stem}" of list 1'],
                            check=True, capture_output=True,
                        )
                        info = plistlib.loads((bundle / "Contents/Info.plist").read_bytes())
                        assert not info["LSUIElement"], info
                        assert (bundle / "Contents/Resources" / info["CFBundleIconFile"]).is_file()
                        subprocess.run(["codesign", "--verify", "--deep", "--strict", str(bundle)],
                                       check=True)
                        state["dockItem"] = True
                        state["bundledStopwatchIcon"] = True
                    assert settings.read_text() == original, "startup changed saved settings"
                    if gui:
                        if hidden:
                            os.killpg(process.pid, signal.SIGINT)
                            process.wait(timeout=5)
                            state["interruptStoppedApp"] = True
                        else:
                            os.kill(pid, signal.SIGTERM)
                            assert process.wait(timeout=5) == 0, "launcher failed after its app exited"
                            state["appExitReturned"] = True
                        deadline = time.monotonic() + 5
                        while app_pid(process.pid, executable) is not None:
                            if time.monotonic() >= deadline:
                                raise AssertionError("dev app remained running after shutdown")
                            time.sleep(0.1)
                        assert not list(launch_tmp.iterdir()), "launcher leaked temporary files"
                        state["launcherWaited"] = True
                        state["temporaryFilesCleaned"] = True
                    print(f"PASS {name}: {json.dumps(state, sort_keys=True)}", flush=True)
                finally:
                    if pid and gui:
                        try:
                            os.kill(pid, signal.SIGTERM)
                        except ProcessLookupError:
                            pass
                    try:
                        os.killpg(process.pid, signal.SIGTERM)
                    except ProcessLookupError:
                        pass
                    process.wait(timeout=5)
                    if gui:
                        register = ("/System/Library/Frameworks/CoreServices.framework/Frameworks/"
                                    "LaunchServices.framework/Support/lsregister")
                        subprocess.run([register, "-u", str(bundle)], capture_output=True)


if __name__ == "__main__":
    main()
