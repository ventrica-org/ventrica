//
//  VNInstallQueue.swift
//  Ventrica
//

import AppKit
import VentricaKit

struct QueueItem: Equatable {
    let name: String
    let version: String
    static func == (lhs: QueueItem, rhs: QueueItem) -> Bool { lhs.name == rhs.name }
}

struct UninstallItem: Equatable {
    let name: String
    let version: String
    static func == (lhs: UninstallItem, rhs: UninstallItem) -> Bool { lhs.name == rhs.name }
}

@MainActor
final class InstallQueue {
	static let shared = InstallQueue()

	private let queue = DispatchQueue(label: "ventrica.installqueue.serial")

	private var _installItems: [QueueItem] = []
	private var _uninstallItems: [UninstallItem] = []
	private var _installedNames: Set<String> = []
	private var _isApplying = false
	private var installedVersions: [String: String] = [:]
	private var reverseDeps: [String: [String]] = [:]

	var installItems: [QueueItem] { queue.sync { _installItems } }
	var uninstallItems: [UninstallItem] { queue.sync { _uninstallItems } }
	var installedNames: Set<String> { queue.sync { _installedNames } }
	var isApplying: Bool { _isApplying }
	var isEmpty: Bool { queue.sync { _installItems.isEmpty && _uninstallItems.isEmpty } }

	private init() {
		_refreshInstalledNames()
		NotificationCenter.default.addObserver(
			self,
			selector: #selector(_appDidBecomeActive),
			name: NSApplication.didBecomeActiveNotification,
			object: nil
		)
	}
	
	func isInstalled(_ name: String) -> Bool {
		queue.sync { _installedNames.contains(name) }
	}
	func isQueued(_ name: String) -> Bool {
		queue.sync { _installItems.contains { $0.name == name } }
	}
	func isQueuedForUninstall(_ name: String) -> Bool {
		queue.sync { _uninstallItems.contains { $0.name == name } }
	}
	
	func enqueue(_ package: Package) {
		queue.sync {
			if _uninstallItems.contains(where: { $0.name == package.name }) {
				_uninstallItems.removeAll { $0.name == package.name }
				DispatchQueue.main.async { self._postChange() }
				return
			}
			
			guard
				!_installedNames.contains(package.name),
				!_installItems.contains(where: { $0.name == package.name })
			else {
				return
			}
			
			_installItems.append(QueueItem(name: package.name, version: package.version))
			DispatchQueue.main.async { self._postChange() }
		}
	}
	
	func enqueueUninstall(_ package: Package) {
		queue.sync {
			if _installItems.contains(where: { $0.name == package.name }) {
				_installItems.removeAll { $0.name == package.name }
				DispatchQueue.main.async { self._postChange() }
				return
			}
			
			guard
				_installedNames.contains(package.name),
				!_uninstallItems.contains(where: { $0.name == package.name })
			else {
				return
			}
			
			_uninstallItems.append(UninstallItem(name: package.name, version: package.version))
			DispatchQueue.main.async { self._postChange() }
		}
	}
	
	func dequeue(_ name: String) {
		queue.sync {
			guard _installItems.contains(where: { $0.name == name }) else { return }
			_installItems.removeAll { $0.name == name }
			DispatchQueue.main.async { self._postChange() }
		}
	}
	
	func dequeueUninstall(_ name: String) {
		queue.sync {
			guard _uninstallItems.contains(where: { $0.name == name }) else { return }
			_uninstallItems.removeAll { $0.name == name }
			DispatchQueue.main.async { self._postChange() }
		}
	}
	
	func clear() {
		queue.sync {
			guard
				!_installItems.isEmpty ||
				!_uninstallItems.isEmpty
			else {
				return
			}
			
			_installItems.removeAll()
			_uninstallItems.removeAll()
			DispatchQueue.main.async { self._postChange() }
		}
	}
	
	// MARK: - Apply
	
	func applyAll(completion: @Sendable @escaping (Bool, String?) -> Void) {
		Task { @MainActor in
			guard !self._isApplying else { return }
			
			guard
				!self._installItems.isEmpty == false ||
				!self._uninstallItems.isEmpty == false
			else {
				completion(true, nil)
				return
			}
			
			self._isApplying = true
			self._postChange()

			let toInstall = self._installItems.map { $0.name }
			let toRemove  = self._uninstallItems.map { ($0.name, $0.version) }

			Task.detached(priority: .userInitiated) { [toInstall, toRemove] in
				let (success, errorMessage) = await Self._applySync(toInstall: toInstall, toRemove: toRemove)
				await MainActor.run {
					self._isApplying = false
					if success {
						self._installItems.removeAll()
						self._uninstallItems.removeAll()
					}
					NotificationCenter.default.post(
						name: .shouldRefreshPackageList,
						object: nil
					)
					self._refreshInstalledNames()
					completion(success, errorMessage)
				}
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
		Task { @MainActor in
			let data = await Self._fetchInstalledData()
			self._installedNames = data.names
			self.installedVersions = data.versions
			self.reverseDeps = data.reverseDeps
			self._postChange()
		}
	}
	
	private struct InstalledData {
		var names: Set<String>
		var versions: [String: String]
		var reverseDeps: [String: [String]]
	}
	
	@MainActor
	private static func _fetchInstalledData() async -> InstalledData {
		return await withCheckedContinuation { continuation in
			DispatchQueue.global(qos: .utility).async {
				var err: UnsafeMutablePointer<VentError>? = nil
				var arr: UnsafeMutablePointer<UnsafeMutablePointer<VentPackage>?>? = nil
				var count: Int = 0
				
				guard ventrica_list_packages(&arr, &count, &err) == 0 else {
					if let e = err { ventrica_error_free(e) }
					continuation.resume(returning: InstalledData(names: [], versions: [:], reverseDeps: [:]))
					return
				}
				
				var names = Set<String>()
				var versions: [String: String] = [:]
				let reverseDeps: [String: [String]] = [:]
				
				if let arr {
					defer { ventrica_pkg_array_free(arr, UInt(bitPattern: count)) }
					for i in 0..<count {
						guard let pkg = arr[i] else { continue }
						let name = String(cString: pkg.pointee.name)
						let version = String(cString: pkg.pointee.version)
						names.insert(name)
						versions[name] = version
					}
				}
				
				continuation.resume(returning: InstalledData(
					names: names,
					versions: versions,
					reverseDeps: reverseDeps
				))
			}
		}
	}
	
	private static func _applySync(
		toInstall: [String],
		toRemove: [(String, String)]
	) -> (Bool, String?) {
		var err: UnsafeMutablePointer<VentError>? = nil
		
		if !toInstall.isEmpty {
			let cArray: [UnsafePointer<CChar>?] = toInstall.map { strdup($0).map { UnsafePointer<CChar>($0) } } + [nil]
			defer { for ptr in cArray where ptr != nil { free(UnsafeMutableRawPointer(mutating: ptr)) } }
			guard ventrica_install(cArray, UInt(toInstall.count), &err) == 0 else {
				return (false, _consumeError(&err))
			}
		}
		if !toRemove.isEmpty {
			let removeNames = toRemove.map { $0.0 }
			let cArray: [UnsafePointer<CChar>?] = removeNames.map { strdup($0).map { UnsafePointer<CChar>($0) } } + [nil]
			defer { for ptr in cArray where ptr != nil { free(UnsafeMutableRawPointer(mutating: ptr)) } }
			guard ventrica_remove(cArray, UInt(toRemove.count), &err) == 0 else {
				return (false, _consumeError(&err))
			}
		}
		return (true, nil)
	}
	
	private static func _consumeError(_ err: inout UnsafeMutablePointer<VentError>?) -> String? {
		guard let e = err else { return nil }
		let msg = String(cString: ventrica_error_message(e))
		ventrica_error_free(e)
		err = nil
		return msg
	}
}
