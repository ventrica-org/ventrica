//
//  VNMainWindowController.swift
//  Ventrica
//
//  Created by samsam on 12/30/25.
//

import AppKit
import VentricaUI

final class VNMainWindowController: NSWindowController {
	init() {
		let window = VNWindow(
			title: Bundle.main.name,
			contentViewController: VNMainSplitViewController()
		)
		
		window.setContentSize(NSSize(width: 1000, height: 700))
		window.contentMinSize = NSSize(width: 1000, height: 300)
		window.titleVisibility = .hidden
		window.titlebarAppearsTransparent = true
		window.styleMask.insert(.fullSizeContentView)
		window.isMovableByWindowBackground = true
		window.toolbarStyle = .unified
		window.toolbar = NSToolbar()
		
		if #available(macOS 15.0, *) {
			window.toolbar?.allowsDisplayModeCustomization = false
		}
		
		super.init(window: window)
	}
	
	@available(*, unavailable)
	required init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}
}
