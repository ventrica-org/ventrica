//
//  VNSectionTableCellView.swift
//  VentricaUI
//
//  Created by samsam on 3/11/26.
//

import AppKit
import SwiftUI

public class VNSectionTableCellView: NSTableCellView {
	public static let identifier = NSUserInterfaceItemIdentifier("VNSectionTableCellView")
	
	private let _titleLabel: NSTextField = {
		let v = NSTextField(labelWithString: "")
		v.font = .systemFont(ofSize: 13, weight: .semibold)
		v.lineBreakMode = .byTruncatingTail
		return v
	}()
	
	override init(frame frameRect: NSRect) {
		super.init(frame: frameRect)
		
		_titleLabel.translatesAutoresizingMaskIntoConstraints = false
		addSubview(_titleLabel)
		
		NSLayoutConstraint.activate([
			_titleLabel.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 8),
			_titleLabel.centerYAnchor.constraint(equalTo: centerYAnchor)
		])
	}
	
	@available(*, unavailable)
	required public init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}
	
	public func configure(title: String) {
		_titleLabel.stringValue = title
	}
}
