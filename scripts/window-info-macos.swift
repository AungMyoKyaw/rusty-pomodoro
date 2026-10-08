// Enumerate one process's windows without activating it; used for size/screenshot verification.
import CoreGraphics
import Foundation
let pid = Int(CommandLine.arguments[1])!
let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as! [[String: Any]]
let mine = windows.filter { ($0[kCGWindowOwnerPID as String] as? Int) == pid }
let data = try JSONSerialization.data(withJSONObject: mine, options: [.prettyPrinted, .sortedKeys])
print(String(data: data, encoding: .utf8)!)
