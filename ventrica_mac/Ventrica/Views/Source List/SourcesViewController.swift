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

final class SourcesViewController: NSViewController {
	private let _scrollView = VNScrollView()
	private var _repoData: [Repo] = []

	init() {
		super.init(nibName: nil, bundle: nil)
	}

	@available(*, unavailable)
	required init?(coder: NSCoder) { fatalError() }

	override func loadView() {
		view = NSView()

		_setupScrollView()
		_setupListeners()
	}

	@objc func addItem(_ sender: Any?) {}
	
	private func _setupScrollView() {
		_scrollView.tableView.delegate = self
		_scrollView.tableView.dataSource = self
		_scrollView.tableView.headerView = nil
		
		view.addSubview(_scrollView)
		
		NSLayoutConstraint.activate([
			_scrollView.topAnchor.constraint(equalTo: view.topAnchor),
			_scrollView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
			_scrollView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
			_scrollView.trailingAnchor.constraint(equalTo: view.trailingAnchor)
		])
	}
	
	private func _setupListeners() {
		NotificationCenter.default.addObserver(
			self,
			selector: #selector(_load),
			name: NSApplication.didBecomeActiveNotification,
			object: nil
		)
		_load()
	}
	
	@objc private func _load() {
		var repos: [Repo] = []
		
		var err: OpaquePointer? = nil
		
		guard let store = ventrica_store_open_default(&err) else {
			if let e = err {
				print(String(cString: ventrica_error_message(e)))
				ventrica_error_free(e)
			}
			return
		}
		defer { ventrica_store_close(store) }
		
		var arr: UnsafeMutablePointer<UnsafeMutablePointer<VentRepo>?>? = nil
		var count: Int = 0
		
		guard ventrica_list_repos(store, &arr, &count, &err) == 0 else {
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
		_scrollView.tableView.reloadData()
	}
}

extension SourcesViewController: NSTableViewDataSource, NSTableViewDelegate {
	func numberOfRows(in tableView: NSTableView) -> Int {
		_repoData.count
	}
	
	func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
		let repo = _repoData[row]
		
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
	
	func tableViewSelectionDidChange(_ notification: Notification) {
		let selectedRow = _scrollView.tableView.selectedRow
		
		guard selectedRow >= 0 else {
			packageDelegate?.viewController(didSelectPackage: nil)
			return
		}
		
		let repo = _repoData[selectedRow]
		
		if let packageDelegate {
			packageDelegate.viewController(didSelectRepo: repo)
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
