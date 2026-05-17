//
//  VNMainWindowController.swift
//  Ventrica
//
//  Created by samsam on 12/30/25.
//

import AppKit
import VentricaUI

final class MainWindowController: NSWindowController {
	private let _splitVC = MainSplitViewController()
	private var _currentContentVC: NSViewController?
	
	private let _toolbar: NSToolbar = {
		let v = NSToolbar(identifier: "VNMainToolbar")
		v.displayMode = .iconOnly
		v.allowsUserCustomization = false
		v.showsBaselineSeparator = true
		if #available(macOS 15.0, *) {
			v.allowsDisplayModeCustomization = false
		}
		return v
	}()
	
	init() {
		let window = VNWindow(
			title: Bundle.main.name,
			contentViewController: _splitVC
		)
		
		window.setContentSize(NSSize(width: 1100, height: 700))
		window.contentMinSize = NSSize(width: 900, height: 300)
		window.titleVisibility = .visible
		window.toolbarStyle = .unified
		
		super.init(window: window)
		
		_splitVC.delegate = self
		
		_toolbar.delegate = self
		window.toolbar = _toolbar
	}
	
	@available(*, unavailable)
	required init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}
	
	private func _toolbarIdentifiers() -> [NSToolbarItem.Identifier] {
		guard let pkgSplit = _currentContentVC as? PackageSplitViewController else {
			return [.toggleSidebar, .mainSeparator]
		}
		return [.toggleSidebar, .mainSeparator, .flexibleSpace, .innerSeparator]
	}
	
	private func _rebuildToolbar() {
		guard let toolbar = window?.toolbar else { return }
		for i in stride(from: toolbar.items.count - 1, through: 0, by: -1) {
			toolbar.removeItem(at: i)
		}
		for (i, id) in _toolbarIdentifiers().enumerated() {
			toolbar.insertItem(withItemIdentifier: id, at: i)
		}
	}
}

extension MainWindowController: MainSplitViewControllerDelegate {
	func splitViewController(_ vc: MainSplitViewController, didSelect controller: NSViewController) {
		_currentContentVC = controller
		_rebuildToolbar()
	}
}

extension MainWindowController: NSToolbarDelegate {
	func toolbarDefaultItemIdentifiers(_ toolbar: NSToolbar) -> [NSToolbarItem.Identifier] {
		_toolbarIdentifiers()
	}
	
	func toolbarAllowedItemIdentifiers(_ toolbar: NSToolbar) -> [NSToolbarItem.Identifier] {
		[.toggleSidebar, .flexibleSpace, .mainSeparator, .innerSeparator]
	}
	
	func toolbar(
		_ toolbar: NSToolbar,
		itemForItemIdentifier itemIdentifier: NSToolbarItem.Identifier,
		willBeInsertedIntoToolbar flag: Bool
	) -> NSToolbarItem? {
		switch itemIdentifier {
		case .mainSeparator:
			return NSTrackingSeparatorToolbarItem(
				identifier: itemIdentifier,
				splitView: _splitVC.splitView,
				dividerIndex: 0
			)
		case .innerSeparator:
			guard let pkgSplit = _currentContentVC as? PackageSplitViewController else { return nil }
			return NSTrackingSeparatorToolbarItem(
				identifier: itemIdentifier,
				splitView: pkgSplit.splitView,
				dividerIndex: 0
			)
		default:
			return nil
		}
	}
}

