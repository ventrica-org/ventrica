//
//  VNMainWindowController.swift
//  Ventrica
//
//  Created by samsam on 12/30/25.
//

import AppKit
import VentricaUI

extension NSToolbarItem.Identifier {
	static let addItem        = NSToolbarItem.Identifier("com.ventrica.toolbar.addItem")
	static let mainSeparator  = NSToolbarItem.Identifier("com.ventrica.toolbar.mainSep")
	static let innerSeparator = NSToolbarItem.Identifier("com.ventrica.toolbar.innerSep")
}

final class MainWindowController: NSWindowController {
	private let _splitVC: VNMainSplitViewController
	private var _currentContentVC: NSViewController?

	init() {
		_splitVC = VNMainSplitViewController()

		let window = VNWindow(
			title: Bundle.main.name,
			contentViewController: _splitVC
		)
		window.setContentSize(NSSize(width: 1100, height: 700))
		window.contentMinSize = NSSize(width: 900, height: 300)
		window.titleVisibility = .visible
		window.toolbarStyle = .unified

		let toolbar = NSToolbar(identifier: "VNMainToolbar")
		toolbar.displayMode = .iconOnly
		toolbar.allowsUserCustomization = false
		toolbar.showsBaselineSeparator = true
		if #available(macOS 15.0, *) {
			toolbar.allowsDisplayModeCustomization = false
		}

		super.init(window: window)

		_splitVC.onContentDidChange = { [weak self] vc in
			self?._currentContentVC = vc
			self?._rebuildToolbar()
		}

		toolbar.delegate = self
		window.toolbar = toolbar
	}

	@available(*, unavailable)
	required init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}

	private func _toolbarIdentifiers() -> [NSToolbarItem.Identifier] {
		guard let pkgSplit = _currentContentVC as? PackageSplitViewController else {
			return [.toggleSidebar, .mainSeparator]
		}
		if pkgSplit.isSourcesList {
			return [.toggleSidebar, .mainSeparator, .addItem, .innerSeparator]
		} else {
			return [.toggleSidebar, .mainSeparator, .flexibleSpace, .innerSeparator]
		}
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

extension MainWindowController: NSToolbarDelegate {
	func toolbarDefaultItemIdentifiers(_ toolbar: NSToolbar) -> [NSToolbarItem.Identifier] {
		_toolbarIdentifiers()
	}

	func toolbarAllowedItemIdentifiers(_ toolbar: NSToolbar) -> [NSToolbarItem.Identifier] {
		[.toggleSidebar, .flexibleSpace, .mainSeparator, .innerSeparator, .addItem]
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
		case .addItem:
			let item = NSToolbarItem(itemIdentifier: itemIdentifier)
			item.image = NSImage(systemSymbolName: "plus", accessibilityDescription: "Add")
			item.label = "Add"
			item.toolTip = "Add"
			return item
		default:
			return nil
		}
	}
}

