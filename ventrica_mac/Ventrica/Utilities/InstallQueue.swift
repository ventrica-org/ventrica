//
//  VNInstallQueue.swift
//  Ventrica
//

import AppKit
import VentricaKit

// MARK: - VNQueueItem

struct QueueItem: Equatable {
	let name: String
	let version: String
	let isDependency: Bool

	static func == (lhs: QueueItem, rhs: QueueItem) -> Bool { lhs.name == rhs.name }
}

// MARK: - VNUninstallItem

struct UninstallItem: Equatable {
	let name: String
	let version: String

	static func == (lhs: UninstallItem, rhs: UninstallItem) -> Bool { lhs.name == rhs.name }
}

final class InstallQueue {
	static let shared = InstallQueue()

	static let didChange = Notification.Name("VNInstallQueueDidChange")

	private(set) var installItems: [QueueItem] = []
	private(set) var uninstallItems: [UninstallItem] = []
	private(set) var installedNames: Set<String> = []
	private(set) var isApplying = false

	var isEmpty: Bool { installItems.isEmpty && uninstallItems.isEmpty }

	private init() {
		_refreshInstalledNames()
		NotificationCenter.default.addObserver(
			self,
			selector: #selector(_appDidBecomeActive),
			name: NSApplication.didBecomeActiveNotification,
			object: nil
		)
	}

	func isInstalled(_ name: String) -> Bool { installedNames.contains(name) }
	func isQueued(_ name: String) -> Bool { installItems.contains { $0.name == name } }
	func isQueuedForUninstall(_ name: String) -> Bool { uninstallItems.contains { $0.name == name } }

	func enqueue(_ package: Package) {
		if isQueuedForUninstall(package.name) {
			uninstallItems.removeAll { $0.name == package.name }
			_postChange()
			return
		}
		guard !isInstalled(package.name), !isQueued(package.name) else { return }

		installItems.append(QueueItem(name: package.name, version: package.version, isDependency: false))
		for dep in package.runDeps where !isInstalled(dep) && !isQueued(dep) {
			installItems.append(QueueItem(name: dep, version: "", isDependency: true))
		}
		_postChange()
	}

	func enqueueUninstall(_ package: Package) {
		if isQueued(package.name) {
			installItems.removeAll { $0.name == package.name }
			_postChange()
			return
		}
		guard isInstalled(package.name), !isQueuedForUninstall(package.name) else { return }

		uninstallItems.append(UninstallItem(name: package.name, version: package.version))
		_postChange()
	}

	func dequeue(_ name: String) {
		guard isQueued(name) else { return }
		installItems.removeAll { $0.name == name }
		_postChange()
	}

	func dequeueUninstall(_ name: String) {
		guard isQueuedForUninstall(name) else { return }
		uninstallItems.removeAll { $0.name == name }
		_postChange()
	}

	func clear() {
		guard !isEmpty else { return }
		installItems.removeAll()
		uninstallItems.removeAll()
		_postChange()
	}

	// MARK: - Apply

	func applyAll(completion: @escaping (Bool, String?) -> Void) {
		guard !isApplying else { return }
		guard !isEmpty else { completion(true, nil); return }

		isApplying = true
		_postChange()

		let toInstall = installItems.map { $0.name }
		let toRemove  = uninstallItems.map { ($0.name, $0.version) }

		DispatchQueue.global(qos: .userInitiated).async { [weak self] in
			let (success, errorMessage) = Self._applySync(toInstall: toInstall, toRemove: toRemove)

			DispatchQueue.main.async { [weak self] in
				self?.isApplying = false
				if success {
					self?.installItems.removeAll()
					self?.uninstallItems.removeAll()
				}
				self?._refreshInstalledNames()
				completion(success, errorMessage)
			}
		}
	}

	func refreshInstalledNames() { _refreshInstalledNames() }

	@objc private func _appDidBecomeActive() {
		_refreshInstalledNames()
	}

	private func _postChange() {
		NotificationCenter.default.post(name: Self.didChange, object: nil)
	}

	private func _refreshInstalledNames() {
		DispatchQueue.global(qos: .utility).async { [weak self] in
			let names = Self._fetchInstalledNames()
			DispatchQueue.main.async { [weak self] in
				self?.installedNames = names
				self?._postChange()
			}
		}
	}

	private static func _fetchInstalledNames() -> Set<String> {
		var err: OpaquePointer? = nil
		guard let store = ventrica_store_open_default(&err) else {
			if let e = err { ventrica_error_free(e) }
			return []
		}
		defer { ventrica_store_close(store) }

		var arr: UnsafeMutablePointer<UnsafeMutablePointer<VentPackage>?>? = nil
		var count: Int = 0

		guard ventrica_list_packages(store, &arr, &count, &err) == 0 else {
			if let e = err { ventrica_error_free(e) }
			return []
		}

		var names = Set<String>()
		if let arr {
			defer { ventrica_pkg_array_free(arr, UInt(count)) }
			for i in 0..<count {
				guard let pkg = arr[i] else { continue }
				names.insert(String(cString: pkg.pointee.name))
			}
		}
		return names
	}

	private static func _applySync(
		toInstall: [String],
		toRemove: [(String, String)]
	) -> (Bool, String?) {
		var err: OpaquePointer? = nil
		guard let store = ventrica_store_open_default(&err) else {
			return (false, _consumeError(&err))
		}
		defer { ventrica_store_close(store) }

		for name in toInstall {
			guard ventrica_install_name(store, name, &err) == 0 else {
				return (false, _consumeError(&err))
			}
		}
		for (name, version) in toRemove {
			guard ventrica_remove(store, name, version, &err) == 0 else {
				return (false, _consumeError(&err))
			}
		}
		return (true, nil)
	}

	private static func _consumeError(_ err: inout OpaquePointer?) -> String? {
		guard let e = err else { return nil }
		let msg = String(cString: ventrica_error_message(e))
		ventrica_error_free(e)
		err = nil
		return msg
	}
}
