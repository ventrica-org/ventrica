//
//  VNInstallQueue.swift
//  Ventrica
//

import AppKit
import VentricaKit

struct QueueItem: Equatable {
	let name: String
	let version: String
	let isDependency: Bool
	
	static func == (lhs: QueueItem, rhs: QueueItem) -> Bool { lhs.name == rhs.name }
}

struct UninstallItem: Equatable {
	let name: String
	let version: String
	let isDependency: Bool
	
	static func == (lhs: UninstallItem, rhs: UninstallItem) -> Bool { lhs.name == rhs.name }
}

final class InstallQueue {
	static let shared = InstallQueue()
	
	private(set) var installItems: [QueueItem] = []
	private(set) var uninstallItems: [UninstallItem] = []
	private(set) var installedNames: Set<String> = []
	private(set) var isApplying = false
	
	private var installedVersions: [String: String] = [:]
	private var reverseDeps: [String: [String]] = [:]

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
		
		// BFS over the reverse-dep map to find every installed package that
		// (transitively) depends on this one
		var bfsQueue = [package.name]
		var visited = Set([package.name])
		var dependents: [UninstallItem] = []
		while !bfsQueue.isEmpty {
			let current = bfsQueue.removeFirst()
			for dependent in reverseDeps[current, default: []] {
				guard !visited.contains(dependent) else { continue }
				visited.insert(dependent)
				if isInstalled(dependent), !isQueuedForUninstall(dependent),
				   let version = installedVersions[dependent] {
					dependents.append(UninstallItem(name: dependent, version: version, isDependency: true))
				}
				bfsQueue.append(dependent)
			}
		}
		dependents.reverse()
		uninstallItems.append(contentsOf: dependents)
		uninstallItems.append(UninstallItem(name: package.name, version: package.version, isDependency: false))
		
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
		let toRemove  = uninstallItems.filter { !$0.isDependency }.map { ($0.name, $0.version) }
		
		DispatchQueue.global(qos: .userInitiated).async { [weak self] in
			let (success, errorMessage) = Self._applySync(toInstall: toInstall, toRemove: toRemove)
			
			DispatchQueue.main.async { [weak self] in
				NotificationCenter.default.post(
					name: .shouldRefreshPackageList,
					object: nil
				)
				
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
		NotificationCenter.default.post(name: .queueDidChange, object: nil)
	}
	
	private func _refreshInstalledNames() {
		DispatchQueue.global(qos: .utility).async { [weak self] in
			let data = Self._fetchInstalledData()
			DispatchQueue.main.async { [weak self] in
				self?.installedNames = data.names
				self?.installedVersions = data.versions
				self?.reverseDeps = data.reverseDeps
				self?._postChange()
			}
		}
	}
	
	private struct InstalledData {
		var names: Set<String>
		var versions: [String: String]
		var reverseDeps: [String: [String]]
	}
	
	private static func _fetchInstalledData() -> InstalledData {
		var err: OpaquePointer? = nil
		guard let store = ventrica_store_open_default(&err) else {
			if let e = err { ventrica_error_free(e) }
			return InstalledData(names: [], versions: [:], reverseDeps: [:])
		}
		defer { ventrica_store_close(store) }
		
		var arr: UnsafeMutablePointer<UnsafeMutablePointer<VentPackage>?>? = nil
		var count: Int = 0
		
		guard ventrica_list_packages(store, &arr, &count, &err) == 0 else {
			if let e = err { ventrica_error_free(e) }
			return InstalledData(names: [], versions: [:], reverseDeps: [:])
		}
		
		var names = Set<String>()
		var versions: [String: String] = [:]
		var reverseDeps: [String: [String]] = [:]
		if let arr {
			defer { ventrica_pkg_array_free(arr, UInt(count)) }
			for i in 0..<count {
				guard let pkg = arr[i] else { continue }
				let name = String(cString: pkg.pointee.name)
				let version = String(cString: pkg.pointee.version)
				names.insert(name)
				versions[name] = version
				let depNames = cStringArrayToSwift(
					pkg.pointee.run_dep_names,
					maxCount: Int(clamping: pkg.pointee.run_dep_names_count)
				)
				for dep in depNames {
					reverseDeps[dep, default: []].append(name)
				}
			}
		}
		return InstalledData(names: names, versions: versions, reverseDeps: reverseDeps)
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
