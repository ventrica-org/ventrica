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
		let vc: NSViewController!
		
		if let package {
			let rootVc = PackageViewController(package: package)
			vc = VNNavigationController(rootViewController: rootVc)
		} else {
			vc = NoPackageViewController()
		}
		
		setDetailViewController(vc)
	}
	
	func viewController(didSelectRepo package: Repo?) {
		let vc: NSViewController!
		
		if let package {
			let rootVc = PackageListViewController(titleText: package.name, url: package.url)
			vc = VNNavigationController(rootViewController: rootVc)
		} else {
			vc = NoPackageViewController()
		}
		
		setDetailViewController(vc)
	}
}
