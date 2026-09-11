import Foundation
import CoreGraphics

let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
let matches = windows.filter { ($0[kCGWindowOwnerName as String] as? String ?? "").lowercased() == "tagryn" && ($0[kCGWindowLayer as String] as? Int ?? 1) == 0 }
guard let window = matches.first, let number = window[kCGWindowNumber as String] as? Int else {
    print("No visible Tagryn window found")
    exit(1)
}
let pid = window[kCGWindowOwnerPID as String] as? Int ?? 0
print("Tagryn window=\(number) pid=\(pid) bounds=\(window[kCGWindowBounds as String] ?? "unknown")")
if CommandLine.arguments.count > 1 {
    let capture = Process()
    capture.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
    capture.arguments = ["-x", "-o", "-l", String(number), CommandLine.arguments[1]]
    try capture.run()
    capture.waitUntilExit()
    exit(capture.terminationStatus)
}
