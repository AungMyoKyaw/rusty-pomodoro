// Observe a specific process without activating another app.
import AppKit
import CoreGraphics
import Foundation
let pid = pid_t(CommandLine.arguments[1])!
guard let app = NSRunningApplication(processIdentifier: pid) else { fatalError("process not running") }
let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as! [[String: Any]]
let mine = windows.filter { ($0[kCGWindowOwnerPID as String] as? Int) == Int(pid) && ($0[kCGWindowName as String] as? String) == "Rusty Pomodoro" }
// Verify the running app's icon is the red/cream stopwatch, not merely a generic non-nil icon.
var customIcon = false
if let image = app.icon, let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil) {
    let bitmap = NSBitmapImageRep(cgImage: cgImage)
    if let tile = bitmap.colorAt(x: bitmap.pixelsWide / 4, y: bitmap.pixelsHigh / 4)?.usingColorSpace(.sRGB),
       let center = bitmap.colorAt(x: bitmap.pixelsWide / 2, y: bitmap.pixelsHigh / 2)?.usingColorSpace(.sRGB) {
        customIcon = tile.redComponent > 0.7 && tile.greenComponent < 0.6 && tile.blueComponent < 0.65
            && tile.alphaComponent > 0.9 && center.redComponent > 0.7 && center.greenComponent > 0.7
            && center.blueComponent > 0.6 && center.alphaComponent > 0.9
    }
}
let result: [String: Any] = [
    "pid": Int(pid),
    "activationPolicy": app.activationPolicy.rawValue,
    "bundleIdentifier": app.bundleIdentifier ?? "",
    "name": app.localizedName ?? "",
    "iconPresent": app.icon != nil,
    "customStopwatchIcon": customIcon,
    "timerVisible": !mine.isEmpty,
    "windowCount": mine.count
]
print(String(data: try JSONSerialization.data(withJSONObject: result, options: [.sortedKeys]), encoding: .utf8)!)
