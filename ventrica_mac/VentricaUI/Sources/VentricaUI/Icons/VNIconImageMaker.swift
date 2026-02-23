//
//  VNIconImageMaker.swift
//  VentricaUI
//
//  Created by samsam on 3/17/26.
//

import AppKit
import SwiftUI

public struct SectionIcon {
	
	nonisolated(unsafe) private static var cache = [String: NSImage]()
	
	private let symbolName: String
	private let color: NSColor
	private let usePrivateSymbol: Bool
	
	public init(symbolName: String, color: NSColor, usePrivateSymbol: Bool = false) {
		self.symbolName = symbolName
		self.color = color
		self.usePrivateSymbol = usePrivateSymbol
	}

	public func image(size: CGSize = CGSize(width: 65, height: 65), symbolScale: CGFloat = 0.80) -> NSImage? {
		let cacheKey = "\(symbolName)-\(color.description)"
		if let cached = SectionIcon.cache[cacheKey] { return cached }

		let image = NSImage(size: size)
		image.lockFocus()
		defer { image.unlockFocus() }
		
		guard let context = NSGraphicsContext.current?.cgContext else { return nil }
		
		let gradientColors = color.appStoreGradientColors().map { $0.cgColor } as CFArray
		if let gradient = CGGradient(colorsSpace: CGColorSpaceCreateDeviceRGB(), colors: gradientColors, locations: [0, 1]) {
			context.drawLinearGradient(
				gradient,
				start: CGPoint(x: size.width/2, y: size.height),
				end: CGPoint(x: size.width/2, y: 0),
				options: []
			)
		}
		var symbol: NSImage? = nil
		if usePrivateSymbol {
			symbol = NSImage.privateImage(systemSymbolName: symbolName)
		}
		if symbol == nil {
			symbol = NSImage(systemSymbolName: symbolName, accessibilityDescription: nil)
		}
		
		if let symbol = symbol {
			var config = NSImage.SymbolConfiguration()
			config = NSImage.SymbolConfiguration(hierarchicalColor: .white)
			let configuredSymbol = symbol.withSymbolConfiguration(config) ?? symbol
			
			let maxDim = min(size.width, size.height) * symbolScale
			let aspect = configuredSymbol.size.width / configuredSymbol.size.height
			let drawSize = CGSize(
				width: maxDim * (aspect > 1 ? 1 : aspect),
				height: maxDim * (aspect > 1 ? 1 / aspect : 1)
			)
			
			let rect = CGRect(
				x: (size.width - drawSize.width) / 2,
				y: (size.height - drawSize.height) / 2,
				width: drawSize.width,
				height: drawSize.height
			)

			configuredSymbol.draw(in: rect, from: .zero, operation: .sourceOver, fraction: 1)
		}

		SectionIcon.cache[cacheKey] = image
		return image
	}
}

extension NSColor {
	func appStoreGradientColors() -> [NSColor] {
		guard let rgb = usingColorSpace(.sRGB) else { return [self, self] }
		let lighter = rgb.blended(withFraction: 0.1, of: .white) ?? rgb
		let darker  = rgb.blended(withFraction: 0.1, of: .black) ?? rgb
		return [lighter, darker]
	}
}

public enum VNCategory: String, CaseIterable {
	case sources,
		 applications,
		 providers,
		 
		 libraries,
		 developer,
		 editors,
		 ai,
		 shells,
		 games,
		 archiving,
		 networking,
		 system,
		 security,
		 media,
		 social,
		 usb

	
	public var symbolName: String {
		switch self {
		case .sources:		"shippingbox.fill"
		case .applications:	"appstore" // usePrivateSymbol
		case .providers:	"storefront.fill"
			
		case .libraries:	"slider.horizontal.3"
		case .developer:	"screwdriver.fill"
		case .editors:		"apple.pages" // usePrivateSymbol
		case .ai:			"sparkle"
		case .shells:		"terminal.fill"
		case .games:		"gamecontroller.fill"
		case .archiving:	"archivebox.fill"
		case .networking:	"wifi"
		case .system:		symbolForDevice()
		case .security:		"lock.fill"
		case .media:		"playpause.fill"
		case .social:		"person.2.fill"
		case .usb:			"port.usb.c" // usePrivateSymbol
		}
	}
	
	public var name: String {
		switch self {
		case .sources:		"Sources"
		case .applications:	"Applications"
		case .providers:	"Providers"
			
		case .libraries:	"Libraries"
		case .developer:	"Developer"
		case .editors:		"Editors"
		case .ai:			"AI"
		case .shells:		"Shells"
		case .games:		"Games"
		case .archiving:	"Archiving"
		case .networking:	"WiFi"
		case .system:		"System"
		case .security:		"Security"
		case .media:		"Media"
		case .social:		"Social Media"
		case .usb:			"USB"
		}
	}
	
	public var usePrivateSymbol: Bool {
		switch self {
		case .editors,
			 .usb,
			 .applications:	true
		default:			false
		}
	}
	
	public var color: NSColor {
		switch self {
		case .sources:		.systemGray
		case .applications:	.systemBlue
		case .providers:	.systemRed
			
		case .libraries:	.systemOrange
		case .developer:	.systemBlue
		case .editors:		.systemTeal
		case .ai:			.systemPink
		case .shells:		.systemGray
		case .games:		.systemPurple
		case .archiving:	.systemBrown
		case .networking:	.systemGreen
		case .system:		.controlBackgroundColor
		case .security:		.systemRed
		case .media:		.systemIndigo
		case .social:		.systemCyan
		case .usb:			.systemGreen
		}
	}
	
}

public struct VNCategoryIdentifier {
	public let category: VNCategory?
	public let subCategory: VNCategory?
	
	public init(_ string: String) {
		let parts = string.split(separator: ".", maxSplits: 1).map(String.init)
		if parts.count == 2 {
			category = VNCategory(rawValue: parts[0])
			subCategory = VNCategory(rawValue: parts[1])
		} else {
			category = VNCategory(rawValue: string)
			subCategory = nil
		}
	}
	
	public var mainSymbolName: String {
		if let cat = category {
			cat.symbolName
		} else {
			"cube.box.fill"
		}
	}
	
	public var iconColor: NSColor {
		if let sub = subCategory {
			return sub.color
		}
		if let cat = category {
			return cat.color
		}
		return .systemGray
	}
	
	public var name: String {
		if let sub = subCategory {
			return sub.name
		}
		if let cat = category {
			return cat.name
		}
		
		return "???"
	}
	
	public var usePrivateSymbol: Bool {
		if let cat = category {
			cat.usePrivateSymbol
		} else {
			false
		}
	}
	
	public var sectionIcon: SectionIcon {
		SectionIcon(symbolName: mainSymbolName, color: iconColor, usePrivateSymbol: usePrivateSymbol)
	}
	
	public var displayName: String {
		if let cat = category, let sub = subCategory { return "\(cat.rawValue).\(sub.rawValue)" }
		if let cat = category { return cat.rawValue }
		return "unknown"
	}
}

// MARK: - Preview

#Preview("VNCategoryIdentifier") {
	let identifiers: [String] = {
		var list: [String] = []
		list += VNCategory.allCases.map { $0.rawValue }
		for category in VNCategory.allCases {
			for sub in VNCategory.allCases {
				list.append("\(category.rawValue).\(sub.rawValue)")
			}
		}
		return list
	}()
		
	ScrollView {
		LazyVGrid(columns: [GridItem(.adaptive(minimum: 90))], spacing: 20) {
			ForEach(identifiers, id: \.self) { identifier in
				let parsed = VNCategoryIdentifier(identifier)
				
				VStack(spacing: 6) {
					if let icon = parsed.sectionIcon.image() {
						Image(nsImage: icon)
							.resizable()
							.frame(width: 65, height: 65)
							.cornerRadius(16)
					}
					
					Text(parsed.displayName)
						.font(.caption2)
						.multilineTextAlignment(.center)
				}
			}
		}
		.padding(30)
	}
	.frame(width: 400, height: 700)
}

private func symbolForDevice() -> String {
	return "macmini.fill"
}
