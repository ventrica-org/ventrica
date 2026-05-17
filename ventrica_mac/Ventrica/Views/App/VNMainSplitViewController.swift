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
		contentVC: VNSidebarSection.discover.makeContentViewController()
	)

	/// Called whenever the active content VC changes. Used by the window controller to rebuild toolbar items.
	var onContentDidChange: ((NSViewController) -> Void)?

	override func viewDidLoad() {
		super.viewDidLoad()

		let sidebar = VNSidebarViewController()
		let sidebarItem = NSSplitViewItem(sidebarWithViewController: sidebar)
		sidebarItem.minimumThickness = 220
		sidebarItem.maximumThickness = 220
		sidebarItem.canCollapse = true
		sidebarItem.canCollapseFromWindowResize = true

		let contentItem = NSSplitViewItem(viewController: _container)

		addSplitViewItem(sidebarItem)
		addSplitViewItem(contentItem)

		splitView.dividerStyle = .thin
	}

	func setContentViewController(_ controller: NSViewController) {
		guard isViewLoaded else { return }
		_container.swapContent(controller)
		view.window?.title = controller.title ?? Bundle.main.name
		onContentDidChange?(controller)
	}
}
