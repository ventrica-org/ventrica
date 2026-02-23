//
//  VNTableView.swift
//  VentricaUI
//
//  Created by samsam on 3/11/26.
//

import AppKit

public class VNScrollView: NSScrollView {
	public let tableView = NSTableView()
	
	public override init(frame frameRect: NSRect) {
		super.init(frame: frameRect)
		_setup()
	}
	
	@available(*, unavailable)
	required public init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}
	
	private func _setup() {
		_setupScrollView()
		_setupTableView()
	}
	
	private func _setupScrollView() {
		translatesAutoresizingMaskIntoConstraints = false
		hasVerticalScroller = true
		drawsBackground = true
	}
	
	private func _setupTableView() {
		tableView.headerView = nil
		tableView.selectionHighlightStyle = .regular
		tableView.usesAlternatingRowBackgroundColors = true
		tableView.rowHeight = 44
		tableView.allowsColumnReordering = false
		tableView.allowsColumnSelection = false
		tableView.allowsColumnResizing = false
		
		let column = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("VNMainColumn"))
		column.resizingMask = .autoresizingMask
		tableView.addTableColumn(column)
		
		documentView = tableView
	}
}
