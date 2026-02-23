//
//  VNWindow.swift
//  VentricaUI
//
//  Created by samsam on 12/27/25.
//

import AppKit

public class VNWindow: NSWindow {
	public init(
		title: String,
		contentViewController: NSViewController
	) {
		super.init(
			contentRect: NSRect.zero,
			styleMask: [
				.titled,
				.closable,
				.resizable,
				.fullSizeContentView,
				.miniaturizable
			],
			backing: .buffered, defer: false
		)
		
		self.center()
		self.title = title
		self.contentViewController = contentViewController
		self.delegate = self
	}
}

// MARK: - NSWindowDelegate

extension VNWindow: NSWindowDelegate {
	public func window(
		_ window: NSWindow,
		willUseFullScreenPresentationOptions proposedOptions: NSApplication.PresentationOptions
	) -> NSApplication.PresentationOptions {
		[.autoHideToolbar, .autoHideMenuBar, .autoHideDock, .fullScreen]
	}
}
