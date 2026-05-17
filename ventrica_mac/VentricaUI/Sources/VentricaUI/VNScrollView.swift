//
//  VNTableView.swift
//  VentricaUI
//
//  Created by samsam on 3/11/26.
//

import AppKit

public class VNScrollView: NSScrollView {
	public let tableView: NSTableView = {
		let v = NSTableView()
		v.headerView = nil
		v.selectionHighlightStyle = .regular
		v.usesAlternatingRowBackgroundColors = true
		v.rowHeight = 44
		v.allowsColumnReordering = false
		v.allowsColumnSelection = false
		v.allowsColumnResizing = false
		v.allowsEmptySelection = false
		return v
	}()
	
	public override init(frame frameRect: NSRect) {
		super.init(frame: frameRect)
		
		translatesAutoresizingMaskIntoConstraints = false
		hasVerticalScroller = true
		drawsBackground = true
		
		let column = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("VNMainColumn"))
		column.resizingMask = .autoresizingMask
		tableView.addTableColumn(column)
		
		documentView = tableView
	}
	
	@available(*, unavailable)
	required public init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}
}
