//
//  VNSidebarViewController.swift
//  Ventrica
//
//  Created by samsam on 12/30/25.
//

import AppKit
import VentricaUI

private class _VNSidebarRowView: NSTableRowView {
	override var isEmphasized: Bool { get { false } set {} }
}

// MARK: - VNSidebarSection
enum VNSidebarSection: CaseIterable {
	case discover,
		 recents,
		 sources,
		 packages,
		 updates
	
	var title: String {
		switch self {
		case .discover: .localized("Discover")
		case .recents:  .localized("Recents")
		case .sources:  .localized("Sources")
		case .packages: .localized("Packages")
		case .updates:  .localized("Updates")
		}
	}
	
	var symbol: String {
		switch self {
		case .discover: "star"
		case .recents:  "clock"
		case .sources: 	"globe.desk"
		case .packages: "shippingbox"
		case .updates:  "square.and.arrow.down"
		}
	}
	
	func makeContentViewController() -> NSViewController {
		switch self {
		case .discover, .recents, .updates:
			let vc = NSViewController()
			vc.title = self.title
			return vc
		case .sources:
			let vc = SourcesViewController()
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

final class VNSidebarViewController: NSViewController {
	private let scrollView = NSScrollView()
	private let outlineView = NSOutlineView()
	private let searchField = NSSearchField()
	
	private let sections = VNSidebarSection.allCases
	private var contentControllers: [VNSidebarSection: NSViewController] = [:]
	private var _initialContentShown = false
	
	private lazy var _searchVC: NSViewController = {
		let vc = NSViewController()
		vc.title = .localized("Search")
		return vc
	}()
	
	private weak var mainSplitViewController: VNMainSplitViewController? {
		parent as? VNMainSplitViewController
	}
	
	init() {
		super.init(nibName: nil, bundle: nil)
		for section in sections {
			contentControllers[section] = section.makeContentViewController()
		}
	}
	
	required init?(coder: NSCoder) { fatalError() }
	
	override func loadView() {
		let v = NSVisualEffectView()
		v.material = .sidebar
		v.blendingMode = .behindWindow
		v.state = .active
		view = v
	}
	
	override func viewDidLoad() {
		super.viewDidLoad()
		setupSearchField()
		setupOutlineView()
		setupLayout()
		
		outlineView.selectRowIndexes(IndexSet(integer: 0), byExtendingSelection: false)
	}
	
	override func viewDidAppear() {
		super.viewDidAppear()
		view.window?.makeFirstResponder(nil)

		guard !_initialContentShown else { return }
		_initialContentShown = true

		DispatchQueue.main.async { [weak self] in
			guard let self = self else { return }
			let initial = self.contentControllers[self.sections[0]]!
			self.showContentController(initial)
		}
	}
	
	private func setupSearchField() {
		searchField.placeholderString = "Search"
		searchField.bezelStyle = .roundedBezel
		searchField.controlSize = .large
		searchField.delegate = self
		searchField.translatesAutoresizingMaskIntoConstraints = false
		view.addSubview(searchField)
	}
	
	private func setupOutlineView() {
		let column = NSTableColumn(identifier: .init("main"))
		outlineView.addTableColumn(column)
		outlineView.outlineTableColumn = column
		outlineView.headerView = nil
		outlineView.style = NSTableView.Style.sourceList
		outlineView.backgroundColor = .clear
		outlineView.rowHeight = 34
		outlineView.rowSizeStyle = .custom
		outlineView.dataSource = self
		outlineView.delegate = self
		
		scrollView.documentView = outlineView
		scrollView.hasVerticalScroller = true
		scrollView.drawsBackground = false
		scrollView.translatesAutoresizingMaskIntoConstraints = false
		view.addSubview(scrollView)
	}
	
	private func setupLayout() {
		NSLayoutConstraint.activate([
			searchField.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor),
			searchField.leadingAnchor.constraint(equalTo: view.leadingAnchor, constant: 9.4),
			searchField.trailingAnchor.constraint(equalTo: view.trailingAnchor, constant: -9.4),
			scrollView.topAnchor.constraint(equalTo: searchField.bottomAnchor, constant: 16),
			scrollView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
			scrollView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
			scrollView.bottomAnchor.constraint(equalTo: view.bottomAnchor)
		])
	}
	
	private func showContentController(_ controller: NSViewController) {
		DispatchQueue.main.async { [weak self] in
			self?.mainSplitViewController?.setContentViewController(controller)
		}
	}
}

// MARK: - Search Delegate
extension VNSidebarViewController: NSSearchFieldDelegate {
	func controlTextDidChange(_ obj: Notification) {
		guard let field = obj.object as? NSSearchField else { return }
		
		if field.stringValue.isEmpty {
			let row = outlineView.selectedRow
			guard row >= 0 else { return }
			let section = sections[row]
			showContentController(contentControllers[section]!)
		} else {
			outlineView.deselectAll(nil)
			showContentController(_searchVC)
		}
	}
}

// MARK: - OutlineView Data Source & Delegate
extension VNSidebarViewController: NSOutlineViewDataSource, NSOutlineViewDelegate {
	func outlineView(_ outlineView: NSOutlineView, numberOfChildrenOfItem item: Any?) -> Int {
		sections.count
	}
	
	func outlineView(_ outlineView: NSOutlineView, isItemExpandable item: Any) -> Bool {
		false
	}
	
	func outlineView(_ outlineView: NSOutlineView, child index: Int, ofItem item: Any?) -> Any {
		sections[index]
	}
	
	func outlineView(_ outlineView: NSOutlineView, viewFor tableColumn: NSTableColumn?, item: Any) -> NSView? {
		guard let section = item as? VNSidebarSection else { return nil }
		
		let cell = NSTableCellView()
		
		let imageView = NSImageView(
			image: NSImage(systemSymbolName: section.symbol, accessibilityDescription: nil)!
		)
		imageView.symbolConfiguration = .init(pointSize: 16, weight: .regular)
		imageView.contentTintColor = .controlAccentColor
		
		let text = NSTextField(labelWithString: section.title)
		text.font = .systemFont(ofSize: 15)
		
		let stack = NSStackView(views: [imageView, text])
		stack.spacing = 8
		stack.alignment = .centerY
		stack.translatesAutoresizingMaskIntoConstraints = false
		cell.addSubview(stack)
		
		NSLayoutConstraint.activate([
			stack.leadingAnchor.constraint(equalTo: cell.leadingAnchor),
			stack.centerYAnchor.constraint(equalTo: cell.centerYAnchor),
			imageView.widthAnchor.constraint(equalToConstant: 20),
			imageView.heightAnchor.constraint(equalToConstant: 20)
		])
		
		return cell
	}
	
	func outlineViewSelectionDidChange(_ notification: Notification) {
		let row = outlineView.selectedRow
		guard row >= 0 else { return }
		let section = sections[row]
		showContentController(contentControllers[section]!)
	}
	
	func outlineView(_ outlineView: NSOutlineView, rowViewForItem item: Any) -> NSTableRowView? {
		_VNSidebarRowView()
	}
}
