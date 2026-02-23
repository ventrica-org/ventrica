//
//  NSImage+private.swift
//  VentricaUI
//
//  Created by samsam on 3/12/26.
//

import AppKit

public extension AppKit.NSImage {
	static func privateImage(systemSymbolName: String) -> NSImage? {
		let selector = Selector(("imageWithPrivateSystemSymbolName:"))
		let image = NSImage.perform(selector, with: systemSymbolName)
		return image?.takeUnretainedValue() as? NSImage
	}
}

