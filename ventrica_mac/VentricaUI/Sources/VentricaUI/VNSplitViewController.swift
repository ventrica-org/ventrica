//
//  VNSplitViewController.swift
//  VentricaUI
//
//  Created by samsam on 5/17/26.
//

import AppKit

open class VNSplitViewController: NSSplitViewController {
	private let _sidebarThickness: CGFloat = 280
	private let _detailMinThickness: CGFloat = 400
	private var _detailItem: NSSplitViewItem!
	private var _isSettingUp = false

	public func setup(listViewController: NSViewController, initialDetailViewController: NSViewController) {
		_isSettingUp = true
		defer { _isSettingUp = false }

		let listItem = NSSplitViewItem(viewController: listViewController)
		listItem.minimumThickness = _sidebarThickness
		listItem.maximumThickness = _sidebarThickness

		_detailItem = NSSplitViewItem(viewController: initialDetailViewController)
		_detailItem.minimumThickness = _detailMinThickness

		addSplitViewItem(_detailItem)
		insertSplitViewItem(listItem, at: 0)

		splitView.dividerStyle = .thin
	}

	public func setDetailViewController(_ controller: NSViewController) {
		guard isViewLoaded, !_isSettingUp else { return }
		
		let newItem = NSSplitViewItem(viewController: controller)
		newItem.minimumThickness = _detailMinThickness
		
		removeSplitViewItem(_detailItem)
		addSplitViewItem(newItem)
		_detailItem = newItem
	}
}
