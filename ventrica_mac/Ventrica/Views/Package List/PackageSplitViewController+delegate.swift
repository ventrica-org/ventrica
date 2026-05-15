//
//  VNPackageSplitViewController+delegate.swift
//  Ventrica
//
//  Created by samsam on 3/18/26.
//

import VentricaUI
import AppKit

// MARK: - VNPackageSplitViewController: VNPackageSplitViewDelegate
extension PackageSplitViewController: PackageSplitViewDelegate {
	func viewController(didSelectPackage package: Package?) {
		let vc: NSViewController = package.map { PackageViewController(package: $0) } ?? NoPackageViewController()
		setDetailViewController(vc)
	}
	
	func viewController(didSelectRepo repo: Repo?) {
		let vc: NSViewController
		if let repo {
			let listVC = PackageListViewController(titleText: repo.name, url: repo.url)
			vc = PackageSplitViewController(listController: listVC)
		} else {
			vc = NoPackageViewController()
		}
		setDetailViewController(vc)
	}
}
