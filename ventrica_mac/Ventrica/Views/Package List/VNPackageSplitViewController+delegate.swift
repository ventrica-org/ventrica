//
//  VNPackageSplitViewController+delegate.swift
//  Ventrica
//
//  Created by samsam on 3/18/26.
//

import VentricaUI

#warning("VNViewController")
// MARK: - VNPackageSplitViewController: VNPackageSplitViewDelegate
extension VNPackageSplitViewController: VNPackageSplitViewDelegate {
	func viewController(didSelectPackage package: VNPackage?) {
		let vc: NSViewController!
		
		if let package {
			let rootVc = VNPackageViewController(package: package)
			vc = VNNavigationController(rootViewController: rootVc)
		} else {
			vc = VNNoPackageViewController()
		}
		
		setDetailViewController(vc)
	}
	
	func viewController(didSelectRepo package: VNRepo?) {
		let vc: NSViewController!
		
		if let package {
			let rootVc = VNPackageListViewController(titleText: package.name, url: package.url)
			vc = VNNavigationController(rootViewController: rootVc)
		} else {
			vc = VNNoPackageViewController()
		}
		
		setDetailViewController(vc)
	}
}
