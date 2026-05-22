//
//  SourcesSplitViewController.swift
//  Ventrica
//
//  Created by samsam on 5/17/26.
//

import AppKit
import VentricaUI

final class SourcesSplitViewController: VNSplitViewController {
	private let _sourcesController = SourcesListViewController()
	
	private let _noSourcesController: EmptyViewController = {
		let v = EmptyViewController()
		v.configure(
			title: .localized("No Source Selected"),
			subtitle: .localized("Select a source from the list to see its details.")
		)
		return v
	}()

	init() {
		super.init(nibName: nil, bundle: nil)
		_sourcesController.delegate = self
	}

	@available(*, unavailable)
	required init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}

	override func viewDidLoad() {
		super.viewDidLoad()
		
		setup(
			listViewController: _sourcesController,
			initialDetailViewController: _noSourcesController
		)
	}
}

extension SourcesSplitViewController: SourcesListViewControllerDelegate {
	func sourcesViewController(_ vc: SourcesListViewController, didSelect repo: Repo?) {
		if let repo {
			setDetailViewController(PackagesCollectionViewController(titleText: repo.name, url: repo.url))
		} else {
			setDetailViewController(_noSourcesController)
		}
	}
}

extension SourcesSplitViewController: ToolbarConfigurable {
	var toolbarIdentifiers: [NSToolbarItem.Identifier] {[
		.toggleSidebar,
		.mainSeparator,
		.flexibleSpace,
		.plus,
		.innerSeparator,
		.flexibleSpace,
		.share
	]}
	
	func performToolbarAction(
		_ identifier: NSToolbarItem.Identifier,
	) {
		switch identifier {
		case .share:	_sourcesController.shareItem()
		case .plus:		_sourcesController.addItem()
		default:		break
		}
	}
	
	func validateToolbarItem(
		_ identifier: NSToolbarItem.Identifier
	) -> Bool {
		switch identifier {
		case .share:	_sourcesController.selectedRepo != nil
		default:		true
		}
	}
}
