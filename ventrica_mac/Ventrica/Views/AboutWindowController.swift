//
//  VNAboutWindowController.swift
//  Ventrica
//
//  Created by samsam on 12/30/25.
//

import AppKit
import VentricaUI

final class AboutWindowController: NSWindowController {
	init() {
		let window = VNWindow(
			title: "About \(Bundle.main.name)",
			contentViewController: AboutViewController()
		)
		
		let size = NSSize(width: 270, height: 320)
		
		window.setContentSize(size)
		window.titlebarAppearsTransparent = true
		window.isMovableByWindowBackground = true
		window.titleVisibility = .hidden
		window.styleMask = [.titled, .closable, .fullSizeContentView]
		window.standardWindowButton(.zoomButton)?.isHidden = true
		window.standardWindowButton(.miniaturizeButton)?.isHidden = true
		
		super.init(window: window)
	}
	
	@available(*, unavailable)
	required init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}
}
