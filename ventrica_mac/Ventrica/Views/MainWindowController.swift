//
//  VNMainWindowController.swift
//  Ventrica
//
//  Created by samsam on 12/30/25.
//

import AppKit
import VentricaUI

final class MainWindowController: NSWindowController {
	private let _contentSize 	= NSSize(width: 1100, height: 700)
	private let _minContentSize = NSSize(width: 900, height: 300)
	
	private let _splitVC = MainSplitViewController()
	
	private weak var _currentContentVC: (
		NSViewController & ToolbarConfigurable
	)?
	
	private let _toolbar: NSToolbar = {
		let v = NSToolbar(identifier: "VNMainToolbar")
		v.displayMode = .iconOnly
		v.allowsUserCustomization = false
		v.showsBaselineSeparator = true
		return v
	}()
	
	init() {
		let window = VNWindow(
			title: Bundle.main.name,
			contentViewController: _splitVC
		)
		
		window.setContentSize(_contentSize)
		window.contentMinSize = _minContentSize
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
}

private extension MainWindowController {
	func _toolbarIdentifiers() -> [NSToolbarItem.Identifier] {
		_currentContentVC?.toolbarIdentifiers ?? [.toggleSidebar, .mainSeparator]
	}
	
	func _rebuildToolbarIfNeeded() {
		guard let toolbar = window?.toolbar else { return }
		
		let current = toolbar.items.map(\.itemIdentifier)
		let desired = _toolbarIdentifiers()
		
		guard current != desired else { return }
		
		for i in stride(
			from: toolbar.items.count - 1,
			through: 0,
			by: -1
		) {
			toolbar.removeItem(at: i)
		}
		
		for (index, identifier) in desired.enumerated() {
			toolbar.insertItem(
				withItemIdentifier: identifier,
				at: index
			)
		}
	}
	
	@objc func _toolbarAction(_ sender: NSToolbarItem) {
		_currentContentVC?.performToolbarAction(sender.itemIdentifier)
	}
}

extension MainWindowController: MainSplitViewControllerDelegate {
	func splitViewController(
		_ vc: MainSplitViewController,
		didSelect controller: NSViewController
	) {
		_currentContentVC = controller
		as? (NSViewController & ToolbarConfigurable)
		
		_rebuildToolbarIfNeeded()
	}
}

extension MainWindowController: NSToolbarDelegate {
	func toolbarDefaultItemIdentifiers(
		_ toolbar: NSToolbar
	) -> [NSToolbarItem.Identifier] {
		_toolbarIdentifiers()
	}
	
	func toolbarAllowedItemIdentifiers(
		_ toolbar: NSToolbar
	) -> [NSToolbarItem.Identifier] {[
		.toggleSidebar,
		.flexibleSpace,
		.mainSeparator,
		.innerSeparator,
		.plus,
		.share,
		.back
	]}
	
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
			guard let splitVC = _currentContentVC as? VNSplitViewController else {
				return nil
			}
			
			return NSTrackingSeparatorToolbarItem(
				identifier: itemIdentifier,
				splitView: splitVC.splitView,
				dividerIndex: 0
			)
		case .plus:
			return _makeToolbarItem(
				identifier: .plus,
				label: "Add",
				symbolName: "plus"
			)
		case .share:
			return _makeToolbarItem(
				identifier: .share,
				label: "Share",
				symbolName: "square.and.arrow.up"
			)
		case .back:
			return _makeToolbarItem(
				identifier: .back,
				label: "Back",
				symbolName: "chevron.left"
			)
		default:
			return nil
		}
	}
}

extension MainWindowController: NSToolbarItemValidation {
	func validateToolbarItem(
		_ item: NSToolbarItem
	) -> Bool {
		_currentContentVC?
			.validateToolbarItem(item.itemIdentifier)
		?? false
	}
}

private extension MainWindowController {
	func _makeToolbarItem(
		identifier: NSToolbarItem.Identifier,
		label: String,
		symbolName: String
	) -> NSToolbarItem {
		
		let item = NSToolbarItem(itemIdentifier: identifier)
		
		item.isBordered = true
		item.label = label
		item.paletteLabel = label
		item.toolTip = label
		
		item.image = NSImage(
			systemSymbolName: symbolName,
			accessibilityDescription: label
		)
		
		item.target = self
		item.action = #selector(_toolbarAction(_:))
		
		return item
	}
}

protocol ToolbarConfigurable: AnyObject {
	var toolbarIdentifiers: [NSToolbarItem.Identifier] { get }
	
	func performToolbarAction(
		_ identifier: NSToolbarItem.Identifier
	)
	
	func validateToolbarItem(
		_ identifier: NSToolbarItem.Identifier
	) -> Bool
}
