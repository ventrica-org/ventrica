//
//  VNPackageSplitViewController.swift
//  Ventrica
//
//  Created by samsam on 12/30/25.
//

import AppKit
import VentricaUI

// MARK: - VNPackageSplitViewController
final class PackageSplitViewController: NSSplitViewController {
	private let _listController: NSViewController
	private var _detailItem: NSSplitViewItem!
	
	init(listController: NSViewController) {
		_listController = listController
		
		super.init(nibName: nil, bundle: nil)
		
		(listController as? SourcesViewController)?.packageDelegate = self
		(listController as? PackageListViewController)?.packageDelegate = self
	}
	
	@available(*, unavailable)
	required init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}
	
	override func viewDidLoad() {
		super.viewDidLoad()
		
		let listItem = NSSplitViewItem(viewController: _listController)
		listItem.minimumThickness = 280
		listItem.maximumThickness = 280
		
		_detailItem = NSSplitViewItem(viewController: NoPackageViewController())
		_detailItem.minimumThickness = 400
		
		addSplitViewItem(listItem)
		addSplitViewItem(_detailItem)
		
		splitView.dividerStyle = .thin
	}
	
	var isSourcesList: Bool { _listController is SourcesViewController }

	func setDetailViewController(_ controller: NSViewController) {
		guard isViewLoaded else {
			return
		}
		
		let newItem = NSSplitViewItem(viewController: controller)
		newItem.minimumThickness = 400
		
		removeSplitViewItem(_detailItem)
		addSplitViewItem(newItem)
		_detailItem = newItem
	}
	// MARK: - Toolbar: forward addItem to list VC if it handles it
	@objc func addItem(_ sender: Any?) {
		(_listController as? SourcesViewController)?.addItem(sender)
	}

	override func validateUserInterfaceItem(_ item: NSValidatedUserInterfaceItem) -> Bool {
		if item.action == #selector(addItem(_:)) {
			return _listController is SourcesViewController
		}
		return super.validateUserInterfaceItem(item)
	}
}

private var _packageDelegateKey: UInt8 = 0

extension NSViewController {
	weak var packageDelegate: PackageSplitViewDelegate? {
		get { objc_getAssociatedObject(self, &_packageDelegateKey) as? PackageSplitViewDelegate }
		set { objc_setAssociatedObject(self, &_packageDelegateKey, newValue, .OBJC_ASSOCIATION_ASSIGN) }
	}
}
