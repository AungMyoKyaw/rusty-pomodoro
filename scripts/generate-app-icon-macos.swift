// Original Tomito Portable stopwatch mark. No proprietary Tomito artwork is used.
// Build with xcrun swiftc -sdk "$(xcrun --show-sdk-path)" -framework AppKit;
// run the resulting executable with <output-directory>.
import AppKit
import Foundation

let output = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)
for size in [16, 32, 64, 128, 256, 512, 1024] {
    let space = CGColorSpace(name: CGColorSpace.sRGB)!
    let canvas = CGContext(data: nil, width: size, height: size, bitsPerComponent: 8,
                           bytesPerRow: size * 4, space: space,
                           bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
    canvas.scaleBy(x: CGFloat(size) / 1024, y: CGFloat(size) / 1024)
    canvas.setShouldAntialias(true)
    canvas.setFillColor(CGColor(red: 232/255, green: 74/255, blue: 95/255, alpha: 1))
    canvas.addPath(CGPath(roundedRect: CGRect(x: 92, y: 92, width: 840, height: 840),
                          cornerWidth: 186, cornerHeight: 186, transform: nil))
    canvas.fillPath()
    let ink = CGColor(red: 1, green: 245/255, blue: 237/255, alpha: 1)
    canvas.setStrokeColor(ink)
    canvas.setFillColor(ink)
    canvas.setLineCap(.round)
    canvas.setLineJoin(.round)
    canvas.setLineWidth(54)
    canvas.strokeEllipse(in: CGRect(x: 268, y: 256, width: 488, height: 488))
    canvas.addPath(CGPath(roundedRect: CGRect(x: 445, y: 766, width: 134, height: 44),
                          cornerWidth: 18, cornerHeight: 18, transform: nil))
    canvas.fillPath()
    canvas.setLineWidth(24)
    for angle in [0.0, Double.pi/2, Double.pi, 3*Double.pi/2] {
        canvas.move(to: CGPoint(x: 512 + cos(angle)*184, y: 500 + sin(angle)*184))
        canvas.addLine(to: CGPoint(x: 512 + cos(angle)*207, y: 500 + sin(angle)*207))
        canvas.strokePath()
    }
    canvas.setLineWidth(48)
    canvas.move(to: CGPoint(x: 512, y: 650))
    canvas.addLine(to: CGPoint(x: 512, y: 500))
    canvas.addLine(to: CGPoint(x: 625, y: 432))
    canvas.strokePath()
    let bitmap = NSBitmapImageRep(cgImage: canvas.makeImage()!)
    let data = bitmap.representation(using: .png, properties: [:])!
    try data.write(to: output.appendingPathComponent("icon-\(size).png"))
}
