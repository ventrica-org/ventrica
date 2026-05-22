//
//  VNSourcesViewController.swift
//  Ventrica
//
//  Created by samsam on 3/8/26.
//

import AppKit
import Combine
import VentricaUI
import VentricaKit

protocol SourcesListViewControllerDelegate: AnyObject {
	func sourcesViewController(_ vc: SourcesListViewController, didSelect repo: Repo?)
}

final class SourcesListViewController: NSViewController {
	weak var delegate: SourcesListViewControllerDelegate?
	
	private let _scrollView = VNScrollView()
	private var _repoData: [Repo] = []
	
	private enum RowItem {
		case section(String)
		case repo(Repo)
	}
	
	private var _rows: [RowItem] = []
	
	var selectedRepo: Repo? {
		let row = _scrollView.tableView.selectedRow
		
		guard row >= 0 else {
			return nil
		}
		
		guard case let .repo(repo) = _rows[row] else {
			return nil
		}
		
		return repo
	}
	
	init() {
		super.init(nibName: nil, bundle: nil)
	}
	
	@available(*, unavailable)
	required init?(coder: NSCoder) { fatalError() }
	
	override func loadView() {
		view = .init()
		
		_setupScrollView()
		_setupListeners()
	}
	
	private func _setupScrollView() {
		_scrollView.tableView.delegate = self
		_scrollView.tableView.dataSource = self
		_scrollView.translatesAutoresizingMaskIntoConstraints = false
		
		view.addSubview(_scrollView)
		
		NSLayoutConstraint.activate([
			_scrollView.topAnchor.constraint(equalTo: view.topAnchor),
			_scrollView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
			_scrollView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
			_scrollView.trailingAnchor.constraint(equalTo: view.trailingAnchor)
		])
	}
	
	private func _setupListeners() {
		_load()
		
		NotificationCenter.default.addObservers(
			[NSApplication.didBecomeActiveNotification, .shouldRefreshSourcesList],
			observer: self,
			selector: #selector(_load)
		)
	}
	
	@objc private func _load() {
		var repos: [Repo] = []
		
		var err: OpaquePointer? = nil
		var arr: UnsafeMutablePointer<UnsafeMutablePointer<VentRepo>?>? = nil
		var count: Int = 0
		
		guard ventrica_list_repos(&arr, &count, &err) == 0 else {
			if let e = err {
				print(String(cString: ventrica_error_message(e)))
				ventrica_error_free(e)
			}
			return
		}
		
		if let arr {
			defer { ventrica_repo_array_free(arr, UInt(count)) }
			for i in 0..<count {
				guard let repo = arr[i] else { continue }
				repos.append(Repo(repo: repo.pointee))
			}
		}
		
		_repoData = repos
		_rebuildRows()
	}
	
	private func _rebuildRows() {
		_rows.removeAll()
		let sorted = _repoData.sorted {
			return $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending
		}
		
		let grouped = Dictionary(grouping: sorted) { pkg -> String in
			return String(pkg.name.prefix(1)).uppercased()
		}
		
		let keys = grouped.keys.sorted { lhs, rhs in
			return lhs < rhs
		}
		
		for key in keys {
			if let pkgs = grouped[key] {
				_rows.append(.section(key))
				pkgs.forEach { _rows.append(.repo($0)) }
			}
		}
		
		_scrollView.tableView.reloadData()
	}
}

extension SourcesListViewController {
	func addItem() {}
	func shareItem() {
		guard
			let selectedRepo,
			let view = view.window?.contentView
		else {
			return
		}
		
		let picker = NSSharingServicePicker(
			items: [selectedRepo.url]
		)
		
		picker.show(
			relativeTo: .zero,
			of: view,
			preferredEdge: .minY
		)
	}
}

extension SourcesListViewController: NSTableViewDataSource, NSTableViewDelegate {
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
		case .repo(let repo):
			let cell = tableView.makeView(
				withIdentifier: VNIconTableCellView.identifier,
				owner: self
			) as? VNIconTableCellView ?? {
				let newCell = VNIconTableCellView()
				newCell.identifier = VNIconTableCellView.identifier
				return newCell
			}()
			
			cell.configure(repo: repo)
			return cell
		}
	}
	
	func tableViewSelectionDidChange(_ notification: Notification) {
		let selectedRow = _scrollView.tableView.selectedRow
		
		guard selectedRow >= 0 else {
			delegate?.sourcesViewController(self, didSelect: nil)
			return
		}

		if case let .repo(pkg) = _rows[selectedRow] {
			delegate?.sourcesViewController(self, didSelect: pkg)
		}
	}
}

extension VentricaUI.VNIconTableCellView {
	func configure(repo: Repo) {
		nameLabel.stringValue = repo.name
		descriptionLabel.stringValue = repo.url
		iconView.image = VNCategoryIdentifier("sources").sectionIcon.image()
	}
}
