//
//  VNAppDelegate.swift
//  Ventrica
//
//  Created by samsam on 12/23/25.
//

import AppKit
import VentricaUI

final class VNAppDelegate: NSObject, NSApplicationDelegate {
	private var _mainWindowController: VNMainWindowController?
	private var _aboutWindowController: VNAboutWindowController?
	
	func applicationDidFinishLaunching(_ notification: Notification) {
		_setupMainMenu()
		_showMainWindow(nil)
	}
	
	func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
		true
	}
	
	private func _setupMainMenu() {
		let mainMenu = NSMenu()
		
		let appMenuItem = NSMenuItem()
		mainMenu.addItem(appMenuItem)
		
		let appMenu = NSMenu()
		appMenuItem.submenu = appMenu
		appMenu.addItem(
			withTitle: "About \(Bundle.main.name)",
			action: #selector(_showAboutWindow(_:)),
			keyEquivalent: ""
		)
		appMenu.addItem(.separator())
		appMenu.addItem(
			withTitle: "Quit \(Bundle.main.name)",
			action: #selector(NSApplication.terminate(_:)),
			keyEquivalent: "q"
		)
		
		NSApp.mainMenu = mainMenu
	}
	
	@objc private func _showMainWindow(_ sender: Any?) {
		if _mainWindowController == nil {
			_mainWindowController = VNMainWindowController()
		}
		
		_mainWindowController?.showWindow(nil)
		_mainWindowController?.window?.center()
		NSApp.activate(ignoringOtherApps: true)
		
		#warning("do onboarding")
//		if !UserDefaults.standard.bool(forKey: "VN.onboarding") {
//			guard let vc = self._mainWindowController?.window?.contentViewController else { return }
//			let onboardingVC = VNOnboardingViewController()
//			vc.presentAsSheet(onboardingVC)
//		}
	}
	
	@objc private func _showAboutWindow(_ sender: Any?) {
		if _aboutWindowController == nil {
			_aboutWindowController = VNAboutWindowController()
		}
		
		_aboutWindowController?.showWindow(nil)
		_aboutWindowController?.window?.makeKeyAndOrderFront(nil)
		NSApp.activate(ignoringOtherApps: true)
	}
}
