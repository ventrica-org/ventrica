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
				.miniaturizable,
				.fullSizeContentView,
			],
			backing: .buffered,
			defer: false,
		)
		
		self.center()
		self.title = title
		self.contentViewController = contentViewController
	}
}
