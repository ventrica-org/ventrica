//
//  VNMainSplitViewController.swift
//  Ventrica
//
//  Created by samsam on 12/24/25.
//

import AppKit
import VentricaUI

enum SidebarSection: CaseIterable {
	case discover, recents, sources, packages, updates
	
	var title: String {
		switch self {
		case .discover:	.localized("Discover")
		case .recents:	.localized("News")
		case .sources:	.localized("Sources")
		case .packages:	.localized("Packages")
		case .updates:	.localized("Updates")
		}
	}
	
	var symbol: String {
		switch self {
		case .discover:	"star"
		case .recents:	"text.book.closed"
		case .sources:	"globe.desk"
		case .packages:	"shippingbox"
		case .updates:	"square.and.arrow.down"
		}
	}
	
	func makeContentViewController() -> NSViewController {
		switch self {
		case .discover, .recents, .updates:
			let vc = NSViewController()
			vc.title = self.title
			return vc
		case .sources:
			let vc = PackageSplitViewController(listController: SourcesViewController())
			vc.title = self.title
			return vc
		case .packages:
			let vc = PackageSplitViewController(
				listController: PackageListViewController(titleText: self.title, url: nil)
			)
			vc.title = self.title
			return vc
		}
	}
}

final class MainSplitViewController: NSSplitViewController {
	weak var delegate: MainSplitViewControllerDelegate?
	
	private let _viewBlur: NSVisualEffectView = {
		let v = NSVisualEffectView()
		v.material = .hudWindow
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
	private var initialContentShown = false
	
	private lazy var searchVC: NSViewController = {
		let v = NSViewController()
		v.title = .localized("Search")
		return v
	}()
	
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
		
		[_sidebarScrollView, _sidebarSearchField].forEach {
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
			_sidebarScrollView.bottomAnchor.constraint(equalTo: _viewBlur.bottomAnchor),
		])
		
		let sidebarVC = NSViewController()
		sidebarVC.view = _viewBlur
		
		let sidebarItem = NSSplitViewItem(sidebarWithViewController: sidebarVC)
		sidebarItem.minimumThickness = 220
		sidebarItem.maximumThickness = 220
		sidebarItem.canCollapse = true
		sidebarItem.canCollapseFromWindowResize = true
		
		[sidebarItem, NSSplitViewItem(viewController: _container)].forEach {
			addSplitViewItem($0)
		}

		_sidebarOutlineView.selectRowIndexes(IndexSet(integer: 0), byExtendingSelection: false)
	}
	
	override func viewDidAppear() {
		super.viewDidAppear()
		view.window?.makeFirstResponder(nil)
		
		guard !initialContentShown else { return }
		initialContentShown = true
		
		DispatchQueue.main.async { [weak self] in
			guard let self = self else { return }
			self.showContentController(self.contentControllers[SidebarSection.allCases[0]]!)
		}
	}
	
	private func showContentController(_ controller: NSViewController) {
		_container.swapContent(controller)
		view.window?.title = controller.title ?? Bundle.main.name
		delegate?.splitViewController(self, didSelect: controller)
	}
}

extension MainSplitViewController: NSOutlineViewDataSource, NSOutlineViewDelegate {
	func outlineView(_ outlineView: NSOutlineView, numberOfChildrenOfItem item: Any?) -> Int {
		SidebarSection.allCases.count
	}
	
	func outlineView(_ outlineView: NSOutlineView, isItemExpandable item: Any) -> Bool {
		false
	}
	
	func outlineView(_ outlineView: NSOutlineView, child index: Int, ofItem item: Any?) -> Any {
		SidebarSection.allCases[index]
	}
	
	func outlineView(
		_ outlineView: NSOutlineView,
		viewFor tableColumn: NSTableColumn?,
		item: Any
	) -> NSView? {
		guard let section = item as? SidebarSection else {
			return nil
		}

		let cell = MainSplitViewCellView()
		cell.configure(with: section)

		return cell
	}
	
	func outlineViewSelectionDidChange(_ notification: Notification) {
		let row = _sidebarOutlineView.selectedRow
		guard row >= 0 else { return }
		showContentController(contentControllers[SidebarSection.allCases[row]]!)
	}
	
	func outlineView(_ outlineView: NSOutlineView, rowViewForItem item: Any) -> NSTableRowView? {
		MainSplitViewRowView()
	}
}

extension MainSplitViewController: NSSearchFieldDelegate {
	func controlTextDidChange(_ obj: Notification) {
		guard let field = obj.object as? NSSearchField else { return }
		
		if field.stringValue.isEmpty {
			let row = _sidebarOutlineView.selectedRow
			guard row >= 0 else { return }
			showContentController(contentControllers[SidebarSection.allCases[row]]!)
		} else {
			_sidebarOutlineView.deselectAll(nil)
			showContentController(searchVC)
		}
	}
}

protocol MainSplitViewControllerDelegate: AnyObject {
	func splitViewController(_ vc: MainSplitViewController, didSelect controller: NSViewController)
}
