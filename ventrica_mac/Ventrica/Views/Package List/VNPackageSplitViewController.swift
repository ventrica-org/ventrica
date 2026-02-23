//
//  VNPackageSplitViewController.swift
//  Ventrica
//
//  Created by samsam on 12/30/25.
//

import AppKit
import VentricaUI

// MARK: - VNPackageSplitViewController
final class VNPackageSplitViewController: NSSplitViewController {
	private let _listController: VNNavigationController
	private var _detailItem: NSSplitViewItem!
	
	init(listController: VNViewController) {
		_listController = VNNavigationController(rootViewController: listController)
		
		super.init(nibName: nil, bundle: nil)
		
		listController.packageDelegate = self
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
		
		_detailItem = NSSplitViewItem(viewController: VNNoPackageViewController())
		_detailItem.minimumThickness = 400
		
		addSplitViewItem(listItem)
		addSplitViewItem(_detailItem)
		
		splitView.dividerStyle = .thin
	}
	
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
}

private var delegateKey: UInt8 = 0

extension VNViewController {
	weak var packageDelegate: VNPackageSplitViewDelegate? {
		get {
			return objc_getAssociatedObject(self, &delegateKey) as? VNPackageSplitViewDelegate
		}
		set {
			objc_setAssociatedObject(self, &delegateKey, newValue, .OBJC_ASSOCIATION_ASSIGN)
		}
	}
}
