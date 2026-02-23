//
//  NSButton++.swift
//  VentricaUI
//
//  Created by samsam on 12/26/25.
//

import AppKit

public extension AppKit.NSView {
	enum VNSemanticStyle: Int {
		case none = 0x0
		case normal = 0x1
		case statusbar = 0x3
		case titlebar = 0x4
		case toolbar = 0x5
		case sourceList = 0x6
		case menu = 0x7
		case unnamed_1 = 0x8
		case unnamed_2 = 0x16
	}
	
	func setSemanticStyle(_ style: VNSemanticStyle) {
		setValue(style.rawValue, forKey: "semanticContext")
	}
}
