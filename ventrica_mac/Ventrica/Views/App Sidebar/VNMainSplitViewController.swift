//
//  VNMainSplitViewController.swift
//  Ventrica
//
//  Created by samsam on 12/24/25.
//

import AppKit
import VentricaUI

final class VNMainSplitViewController: NSSplitViewController {
	private let _container = ContentContainerViewController(
		contentVC: VNSidebarSection.discover.makeNavigationController()
	)

	override func viewDidLoad() {
		super.viewDidLoad()

		let sidebar = VNSidebarViewController()
		let sidebarItem = NSSplitViewItem(sidebarWithViewController: sidebar)
		sidebarItem.minimumThickness = 220
		sidebarItem.maximumThickness = 220
		sidebarItem.canCollapse = false
		sidebarItem.canCollapseFromWindowResize = true

		let contentItem = NSSplitViewItem(viewController: _container)

		addSplitViewItem(sidebarItem)
		addSplitViewItem(contentItem)

		splitView.dividerStyle = .thin
	}

	func setContentViewController(_ controller: NSViewController) {
		guard isViewLoaded else { return }
		_container.swapContent(controller)
	}
}
