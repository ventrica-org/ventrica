//
//  VNMainSplitViewController.swift
//  Ventrica
//
//  Created by samsam on 12/24/25.
//

import AppKit
import VentricaUI

final class VNMainSplitViewController: NSSplitViewController {
	private(set) var contentItem: NSSplitViewItem!
	
	override func viewDidLoad() {
		super.viewDidLoad()
		
		let sidebar = VNSidebarViewController()
		let sidebarItem = NSSplitViewItem(sidebarWithViewController: sidebar)
		sidebarItem.minimumThickness = 220
		sidebarItem.maximumThickness = 220
		sidebarItem.canCollapse = false
		sidebarItem.canCollapseFromWindowResize = true
		
		let initialVC = VNSidebarSection.discover.makeNavigationController()
		contentItem = NSSplitViewItem(viewController: initialVC)
		
		addSplitViewItem(sidebarItem)
		addSplitViewItem(contentItem)
		
		splitView.dividerStyle = .thin
	}
	
	func setContentViewController(_ controller: NSViewController) {
		guard
			isViewLoaded,
			contentItem.viewController !== controller
		else {
			return
		}
		
		let newItem = NSSplitViewItem(viewController: controller)
		
		removeSplitViewItem(contentItem)
		addSplitViewItem(newItem)
		
		contentItem = newItem
	}
}
