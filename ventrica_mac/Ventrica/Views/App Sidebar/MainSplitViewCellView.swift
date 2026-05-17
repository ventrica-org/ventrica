//
//  MainSplitViewRowView.swift
//  Ventrica
//
//  Created by samsam on 5/17/26.
//


import AppKit

final class MainSplitViewRowView: NSTableRowView {
	override var isEmphasized: Bool { get { false } set {} }
}

final class MainSplitViewCellView: NSTableCellView {
	private let _iconView: NSImageView = {
		let v = NSImageView()
		v.symbolConfiguration = .init(
			pointSize: 16,
			weight: .regular
		)
		v.contentTintColor = .controlAccentColor
		return v
	}()
	
	private let _titleLabel: NSTextField = {
		let v = NSTextField(labelWithString: "")
		v.font = .systemFont(ofSize: 15)
		return v
	}()
	
	private lazy var _contentStack: NSStackView = {
		let v = NSStackView(views: [_iconView, _titleLabel])
		v.orientation = .horizontal
		v.spacing = 8
		v.alignment = .centerY
		v.translatesAutoresizingMaskIntoConstraints = false
		return v
	}()
	
	override init(frame frameRect: NSRect) {
		super.init(frame: frameRect)

		addSubview(_contentStack)
		
		NSLayoutConstraint.activate([
			_contentStack.leadingAnchor.constraint(equalTo: leadingAnchor),
			_contentStack.centerYAnchor.constraint(equalTo: centerYAnchor),
			_iconView.widthAnchor.constraint(equalToConstant: 20),
			_iconView.heightAnchor.constraint(equalToConstant: 20),
		])
	}
	
	@available(*, unavailable)
	required public init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}
	
	func configure(with section: SidebarSection) {
		_iconView.image = NSImage(
			systemSymbolName: section.symbol,
			accessibilityDescription: nil
		)
		
		_titleLabel.stringValue = section.title
	}
}
