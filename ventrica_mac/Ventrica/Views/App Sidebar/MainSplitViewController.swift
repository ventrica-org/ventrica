//
//  VNMainSplitViewController.swift
//  Ventrica
//
//  Created by samsam on 12/24/25.
//

import AppKit
import VentricaUI
import VentricaKit

enum SidebarSection: CaseIterable {
	case discover, recents, packages, updates
	
	var title: String {
		switch self {
		case .discover: .localized("Discover")
		case .recents: .localized("News")
		case .packages: .localized("Packages")
		case .updates: .localized("Updates")
		}
	}
	
	var symbol: String {
		switch self {
		case .discover: "star"
		case .recents: "text.book.closed"
		case .packages: "shippingbox"
		case .updates: "square.and.arrow.down"
		}
	}
	
	func makeContentViewController() -> NSViewController {
		switch self {
		case .discover, .recents, .updates:
			let vc = NSViewController()
			vc.title = self.title
			return vc
			
		case .packages:
			let vc = PackagesSplitViewController(
				titleText: self.title,
				url: nil
			)
			vc.title = self.title
			return vc
		}
	}
}

enum SidebarSectionType: Int, CaseIterable {
	case repositories
	
	var title: String {
		switch self {
		case .repositories: "My Repos"
		}
	}
}

enum SidebarItem {
	case section(SidebarSectionType)
	case navigation(SidebarSection)
	case repo(Repo)
}

final class MainSplitViewController: NSSplitViewController {
	weak var delegate: MainSplitViewControllerDelegate?
	
	private let _viewBlur: NSVisualEffectView = {
		let v = NSVisualEffectView()
		v.material = .sidebar
		v.blendingMode = .behindWindow
		v.state = .active
		return v
	}()
	
	private let _container = ContentContainerViewController(
		contentVC: SidebarSection.discover.makeContentViewController()
	)
	
	private let _sidebarScrollView: NSScrollView = {
		let v = NSScrollView()
		v.drawsBackground = false
		v.hasVerticalScroller = false
		return v
	}()
	
	private let _sidebarOutlineView: NSOutlineView = {
		let v = NSOutlineView()
		v.style = .sourceList
		v.backgroundColor = .clear
		v.rowHeight = 34
		v.rowSizeStyle = .custom
		v.headerView = nil
		return v
	}()
	
	private let _sidebarSearchField: NSSearchField = {
		let v = NSSearchField()
		v.placeholderString = .localized("Search")
		v.bezelStyle = .roundedBezel
		v.controlSize = .large
		return v
	}()
	
	private var contentControllers: [SidebarSection: NSViewController] = [:]
	private var _rows: [SidebarItem] = []
	private var _repoData: [Repo] = []
	
	override func viewDidLoad() {
		super.viewDidLoad()
		
		splitView.dividerStyle = .thin
		
		for section in SidebarSection.allCases {
			contentControllers[section] = section.makeContentViewController()
		}
		
		let column = NSTableColumn(identifier: .init("main"))
		_sidebarOutlineView.addTableColumn(column)
		_sidebarOutlineView.outlineTableColumn = column
		_sidebarOutlineView.dataSource = self
		_sidebarOutlineView.delegate = self
		
		_sidebarScrollView.documentView = _sidebarOutlineView
		
		let addRepoButton = NSButton(
			title: "Add Repo",
			image: .init(systemSymbolName: "plus.circle", accessibilityDescription: nil)!,
			target: self,
			action: #selector(_addRepoAction(_:))
		)
		addRepoButton.isBordered = false
		
		[_sidebarScrollView, _sidebarSearchField, addRepoButton].forEach {
			$0.translatesAutoresizingMaskIntoConstraints = false
			_viewBlur.addSubview($0)
		}
		
		_sidebarSearchField.delegate = self
		
		NSLayoutConstraint.activate([
			_sidebarSearchField.topAnchor.constraint(equalTo: _viewBlur.safeAreaLayoutGuide.topAnchor),
			_sidebarSearchField.leadingAnchor.constraint(equalTo: _viewBlur.leadingAnchor, constant: 9.4),
			_sidebarSearchField.trailingAnchor.constraint(equalTo: _viewBlur.trailingAnchor, constant: -9.4),
			
			_sidebarScrollView.topAnchor.constraint(equalTo: _sidebarSearchField.bottomAnchor, constant: 16),
			_sidebarScrollView.leadingAnchor.constraint(equalTo: _viewBlur.leadingAnchor),
			_sidebarScrollView.trailingAnchor.constraint(equalTo: _viewBlur.trailingAnchor),
			_sidebarScrollView.bottomAnchor.constraint(equalTo: addRepoButton.topAnchor),
			
			addRepoButton.leadingAnchor.constraint(equalTo: _viewBlur.leadingAnchor, constant: 9.4),
			addRepoButton.bottomAnchor.constraint(equalTo: _viewBlur.bottomAnchor, constant: -9.4),
		])
		
		let sidebarVC = NSViewController()
		sidebarVC.view = _viewBlur
		
		let sidebarItem = NSSplitViewItem(
			sidebarWithViewController: sidebarVC
		)
		
		sidebarItem.minimumThickness = 220
		sidebarItem.maximumThickness = 220
		sidebarItem.canCollapse = true
		sidebarItem.canCollapseFromWindowResize = true
		
		[sidebarItem, NSSplitViewItem(viewController: _container)].forEach {
			addSplitViewItem($0)
		}
		
		_setupListeners()
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
		_rows.append(.navigation(.discover))
		_rows.append(.navigation(.recents))
		_rows.append(.navigation(.packages))
		_rows.append(.navigation(.updates))
		
		if !_repoData.isEmpty {
			_rows.append(.section(.repositories))
			
			let sorted = _repoData.sorted {
				$0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending
			}
			
			_rows.append(contentsOf: sorted.map { .repo($0) })
		}
		
		_sidebarOutlineView.reloadData()
	}
	
	private func _showContentController(_ controller: NSViewController) {
		_container.swapContent(controller)
		
		view.window?.title =
		controller.title ??
		Bundle.main.name
		
		delegate?.splitViewController(
			self,
			didSelect: controller
		)
	}
	
	@objc private func _addRepoAction(_ sender: NSButton?) {
		let alert = NSAlert()
		alert.messageText = "Add Repository"
		alert.informativeText = "Enter the repository URL or name."

		alert.addButton(withTitle: "Add")
		alert.addButton(withTitle: "Cancel")

		let textField = NSTextField(frame: NSRect(x: 0, y: 0, width: 300, height: 24))
		textField.placeholderString = "Repository URL"

		alert.accessoryView = textField

		guard let window = self.view.window else { return }

		alert.beginSheetModal(for: window) { response in
			if response == .alertFirstButtonReturn {
				let value = textField.stringValue
				var err: OpaquePointer? = nil
				
				guard ventrica_add_repo(value, &err) == 0 else {
					if let e = err {
						print(String(cString: ventrica_error_message(e)))
						ventrica_error_free(e)
					}
					return
				}
				
				NotificationCenter.default.post(
					name: .shouldRefreshSourcesList,
					object: nil
				)
			}
		}
	}
}

extension MainSplitViewController: NSOutlineViewDataSource, NSOutlineViewDelegate {
	func outlineView(_ outlineView: NSOutlineView, numberOfChildrenOfItem item: Any?) -> Int {
		item == nil ? _rows.count : 0
	}
	
	func outlineView(_ outlineView: NSOutlineView, child index: Int, ofItem item: Any?) -> Any {
		_rows[index]
	}
	
	func outlineView(_ outlineView: NSOutlineView, isItemExpandable item: Any) -> Bool {
		false
	}
	
	func outlineView(
		_ outlineView: NSOutlineView,
		viewFor tableColumn: NSTableColumn?,
		item: Any
	) -> NSView? {
		guard let item = item as? SidebarItem else { return nil }

		switch item {
		case .section(let sectionType):
			let cell = VNSectionTableCellView()
			cell.configure(
				title: sectionType.title,
				color: .secondaryLabelColor,
				fontSize: 11
			)
			return cell
		case .navigation(let section):
			let cell = MainSplitViewCellView()
			cell.configure(with: section)
			return cell
		case .repo(let repo):
			let cell = MainSplitViewCellView()
			cell.configure(with: repo)
			return cell
		}
	}
	
	func outlineViewSelectionDidChange(_ notification: Notification) {
		let row = _sidebarOutlineView.selectedRow
		guard row >= 0, row < _rows.count else { return }
		
		let item = _rows[row]
		
		switch item {
		case .section:
			return
		case .navigation(let section):
			_showContentController(contentControllers[section]!)
		case .repo(let repo):
			let vc = PackagesSplitViewController(
				titleText: repo.name,
				url: repo.url
			)
			
			vc.title = repo.name
			_showContentController(vc)
		}
	}
	
	func outlineView(_ outlineView: NSOutlineView, isGroupItem item: Any) -> Bool {
		guard let item = item as? SidebarItem else { return false }
		
		if case .section = item {
			return true
		}
		
		return false
	}
	
	func outlineView(_ outlineView: NSOutlineView, shouldSelectItem item: Any) -> Bool {
		guard let item = item as? SidebarItem else {
			return true
		}
		
		if case .section = item {
			return false
		}
		
		return true
	}
	
	func outlineView(
		_ outlineView: NSOutlineView,
		rowViewForItem item: Any
	) -> NSTableRowView? {
		MainSplitViewRowView()
	}
}

extension MainSplitViewController: NSSearchFieldDelegate {
	func controlTextDidChange(_ obj: Notification) {
		guard let field = obj.object as? NSSearchField else {
			return
		}
		
		if field.stringValue.isEmpty {
			let row = _sidebarOutlineView.selectedRow
			guard row >= 0, row < _rows.count else {
				return
			}
			
			switch _rows[row] {
			case .section:
				return
			case .navigation(let section):
				_showContentController(contentControllers[section]!)
			case .repo(let repo):
				let vc = PackagesSplitViewController(
					titleText: repo.name,
					url: repo.url
				)
				
				vc.title = repo.name
				_showContentController(vc)
			}
		} else {
			_sidebarOutlineView.deselectAll(nil)
			return
		}
	}
}

protocol MainSplitViewControllerDelegate: AnyObject {
	func splitViewController(
		_ vc: MainSplitViewController,
		didSelect controller: NSViewController
	)
}
