//
//  aaa.swift
//  Ventrica
//
//  Created by samsam on 5/17/26.
//

import AppKit

extension NSNotification.Name {
	static let shouldRefreshPackageList = Notification.Name("VN.shouldRefreshPackageList")
	static let shouldRefreshSourcesList = Notification.Name("VN.shouldRefreshSourcesList")
	static let queueDidChange			= Notification.Name("VN.queueDidChange")
}

extension NotificationCenter {
	func addObservers(
		_ notifications: [NSNotification.Name],
		for object: Any? = nil,
		observer: Any,
		selector: Selector
	) {
		notifications.forEach { name in
			self.addObserver(observer, selector: selector, name: name, object: object)
		}
	}
}
