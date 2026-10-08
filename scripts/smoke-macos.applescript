-- Run against a fresh app using an isolated RUSTY_POMODORO_CONFIG_DIR.
-- Usage: osascript scripts/smoke-macos.applescript <pid>
on run argv
    set processId to (item 1 of argv) as integer
    tell application "System Events"
        set appProcess to first application process whose unix id is processId
        tell appProcess
            click button "Stop" of window "Rusty Pomodoro"
            set initialTime to value of static text 2 of window "Rusty Pomodoro"
            if initialTime is not "25:00" then error "Expected a fresh idle session"
            key code 49 using {control down, option down}
            delay 2
            set runningTime to value of static text 2 of window "Rusty Pomodoro"
            if runningTime is initialTime then error "Global start hotkey did not start timer"
            key code 49 using {control down, option down}
            delay 0.3
            set pausedTime to value of static text 2 of window "Rusty Pomodoro"
            delay 1.2
            if value of static text 2 of window "Rusty Pomodoro" is not pausedTime then error "Pause did not freeze timer"
            key code 15 using {control down, option down}
            delay 0.3
            if value of static text 2 of window "Rusty Pomodoro" is not "25:00" then error "Restart hotkey failed"
            key code 1 using {control down, option down}
            delay 0.3
            if value of static text 1 of window "Rusty Pomodoro" is not "Short Break" then error "Skip did not select short break"
            key code 7 using {control down, option down}
            delay 0.3
            if value of static text 2 of window "Rusty Pomodoro" is not "05:00" then error "Stop hotkey failed"
            click (first button whose subrole is "AXCloseButton") of window "Rusty Pomodoro"
            delay 0.3
            if exists window "Rusty Pomodoro" then error "Closing timer did not hide it"
            key code 17 using {control down, option down}
            delay 0.3
            if not (exists window "Rusty Pomodoro") then error "Global show hotkey did not restore window"
            click button "Settings" of window "Rusty Pomodoro"
            delay 0.3
            set value of text field 1 of scroll area 1 of window "Settings" to "9999"
            click button "Apply and save" of window "Settings"
            if value of text field 1 of scroll area 1 of window "Settings" is not "180" then error "Duration was not clamped"
            click button "Statistics" of window "Rusty Pomodoro"
            delay 0.3
            if not (exists window "Statistics") then error "Statistics did not open"
            click button "Export CSV" of window "Statistics"
        end tell
    end tell
    return "PASS: start/pause/restart/skip/stop global hotkeys; hide/restore; settings clamp/save; statistics/export"
end run
