//
//  VNPackageSplitViewController.swift
//  Ventrica
//
//  Created by samsam on 12/30/25.
//

import AppKit
import VentricaUI

final class PackagesSplitViewController: VNSplitViewController {
	private let _packagesController: PackagesListViewController
	
	private let _noPackagesController: EmptyViewController = {
		let v = EmptyViewController()
		v.configure(
			title: .localized("No Package Selected"),
			subtitle: .localized("Select a package from the list to see its details.")
		)
		return v
	}()
	
	init(titleText: String, url: String?) {
		_packagesController = PackagesListViewController(titleText: titleText, url: url)
		super.init(nibName: nil, bundle: nil)
		_packagesController.delegate = self
	}
	
	@available(*, unavailable)
	required init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}
	
	override func viewDidLoad() {
		super.viewDidLoad()
		
		setup(
			listViewController: _packagesController,
			initialDetailViewController: _noPackagesController
		)
	}
}

extension PackagesSplitViewController: PackagesListViewControllerDelegate {
	func packageListViewController(_ vc: PackagesListViewController, didSelect package: Package?) {
		if let package {
			setDetailViewController(PackageViewController(package: package))
		} else {
			setDetailViewController(_noPackagesController)
		}
	}
}
