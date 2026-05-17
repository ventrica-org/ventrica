//
//  VNPackagesViewController.swift
//  Ventrica
//
//  Created by samsam on 12/30/25.
//

import AppKit
import VentricaUI
import Combine
import VentricaKit

// MARK: - VNPackageSplitViewDelegate
protocol PackageSplitViewDelegate: AnyObject {
	func viewController(didSelectPackage package: Package?)
	func viewController(didSelectRepo package: Repo?)
}

// MARK: - VNPackagesViewController
final class PackageListViewController: NSViewController {
	private let _scrollView = VNScrollView()
	private var _packageData: [Package] = []
	private var _url: String?
	
	private enum RowItem {
		case section(String)
		case package(Package)
	}
	
	private var _rows: [RowItem] = []
	
	init(titleText: String, url: String?) {
		self._url = url
		super.init(nibName: nil, bundle: nil)
		self.title = titleText
	}
	
	@available(*, unavailable)
	required init?(coder: NSCoder) { fatalError() }
	
	override func loadView() {
		view = NSView()
		
		_setupScrollView()
		_setupListeners()
	}
	
	private func _setupScrollView() {
		_scrollView.tableView.delegate = self
		_scrollView.tableView.dataSource = self
		_scrollView.tableView.headerView = nil
		_scrollView.tableView.allowsEmptySelection = false
		
		_scrollView.translatesAutoresizingMaskIntoConstraints = false
		view.addSubview(_scrollView)
		
		NSLayoutConstraint.activate([
			_scrollView.topAnchor.constraint(equalTo: view.topAnchor),
			_scrollView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
			_scrollView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
			_scrollView.trailingAnchor.constraint(equalTo: view.trailingAnchor)
		])
	}
	
	private func _rebuildRows() {
		_rows.removeAll()
		
		let sorted = _packageData.sorted {
			if _url != nil {
				return $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending
			} else {
				return $0.addedAt! > $1.addedAt!
			}
		}
		
		let dateFormatter = DateFormatter()
		dateFormatter.dateStyle = .medium
		dateFormatter.timeStyle = .none
		
		let grouped = Dictionary(grouping: sorted) { pkg -> String in
			if _url != nil {
				return String(pkg.name.prefix(1)).uppercased()
			} else {
				let date = Date(timeIntervalSince1970: TimeInterval(pkg.addedAt!))
				return dateFormatter.string(from: date)
			}
		}
		
		let keys = grouped.keys.sorted { lhs, rhs in
			if _url != nil {
				return lhs < rhs
			} else {
				if
					let lhsDate = dateFormatter.date(from: lhs),
					let rhsDate = dateFormatter.date(from: rhs)
				{
					return lhsDate > rhsDate
				}
				return false
			}
		}
		
		for key in keys {
			if let pkgs = grouped[key] {
				_rows.append(.section(key))
				pkgs.forEach { _rows.append(.package($0)) }
			}
		}
		
		_scrollView.tableView.reloadData()
	}
	
	private func _setupListeners() {
		_load()
		NotificationCenter.default.addObserver(
			self,
			selector: #selector(_load),
			name: NSApplication.didBecomeActiveNotification,
			object: nil
		)
	}
	
	@objc private func _load() {
		var packages: [Package] = []
		var err: OpaquePointer? = nil
		
		guard let store = ventrica_store_open_default(&err) else {
			if let e = err {
				print(String(cString: ventrica_error_message(e)))
				ventrica_error_free(e)
			}
			
			return
		}
		
		defer {
			ventrica_store_close(store)
		}
		
		if let url = _url {
			print(url)
			var pkgArr: UnsafeMutablePointer<UnsafeMutablePointer<VentRepoPackage>?>? = nil
			var pkgCount: Int = 0
			
			guard ventrica_list_repo_packages(store, url, &pkgArr, &pkgCount, &err) == 0 else {
				if let e = err {
					print(String(cString: ventrica_error_message(e)))
					ventrica_error_free(e)
				}
				
				return
			}
			
			if let pkgArr {
				defer {
					ventrica_repo_package_array_free(pkgArr, UInt(pkgCount))
				}
				
				for i in 0..<pkgCount {
					guard let pkg = pkgArr[i] else { continue }
					packages.append(Package(repoPackage: pkg.pointee))
				}
			}
		} else {
			var arr: UnsafeMutablePointer<UnsafeMutablePointer<VentPackage>?>? = nil
			var count: Int = 0
			
			guard ventrica_list_packages(store, &arr, &count, &err) == 0 else {
				if let e = err {
					print(String(cString: ventrica_error_message(e)))
					ventrica_error_free(e)
				}
				
				return
			}
			
			if let arr {
				defer {
					ventrica_pkg_array_free(arr, UInt(count))
				}
				
				for i in 0..<count {
					guard let pkg = arr[i] else { continue }
					packages.append(Package(package: pkg.pointee))
				}
			}
		}
		
		_packageData = packages
		_rebuildRows()
		
	}
}

// MARK: - VNPackagesViewController & DataSource

extension PackageListViewController: NSTableViewDataSource, NSTableViewDelegate {
	func numberOfRows(in tableView: NSTableView) -> Int {
		_rows.count
	}
	
	func tableView(_ tableView: NSTableView, isGroupRow row: Int) -> Bool {
		if case .section = _rows[row] {
			true
		} else {
			false
		}
	}
	
	func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
		switch _rows[row] {
		case .section(let title):
			let cell = tableView.makeView(
				withIdentifier: VNSectionTableCellView.identifier,
				owner: self
			) as? VNSectionTableCellView ?? {
				let newCell = VNSectionTableCellView()
				newCell.identifier = VNSectionTableCellView.identifier
				return newCell
			}()
			
			cell.configure(title: title)
			return cell
		case .package(let package):
			let cell = tableView.makeView(
				withIdentifier: VNIconTableCellView.identifier,
				owner: self
			) as? VNIconTableCellView ?? {
				let newCell = VNIconTableCellView()
				newCell.identifier = VNIconTableCellView.identifier
				return newCell
			}()
			
			cell.configure(package: package)
			return cell
		}
	}
	
	func tableViewSelectionDidChange(_ notification: Notification) {
		let selectedRow = _scrollView.tableView.selectedRow
		
		guard selectedRow >= 0 else {
			packageDelegate?.viewController(didSelectPackage: nil)
			return
		}
		
		if case let .package(pkg) = _rows[selectedRow] {
			packageDelegate?.viewController(didSelectPackage: pkg)
		}
	}
}

extension VentricaUI.VNIconTableCellView {
	func configure(package: Package) {
		nameLabel.stringValue = package.name
		descriptionLabel.stringValue = "\(package.version) • \(package.description)"
		
		iconView.image = VNCategoryIdentifier(package.category).sectionIcon.image()
		
		if let iconString = package.icon, let url = URL(string: iconString) {
			Task {
				if let image = await ImageLoader.shared.load(url: url) {
					await MainActor.run {
						self.iconView.image = image
					}
				}
			}
		}
	}
}
