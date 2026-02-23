//
//  VNSeperatorView.swift
//  VentricaUI
//
//  Created by samsam on 3/17/26.
//

import AppKit

final public class VNSeperatorView: NSView {
	override init(frame frameRect: NSRect) {
		super.init(frame: frameRect)
		_setup()
	}
	
	@available(*, unavailable)
	required public init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}
	
	private func _setup() {
		wantsLayer = true
		layer?.backgroundColor = NSColor.separatorColor.cgColor
		NSLayoutConstraint.activate([
			heightAnchor.constraint(equalToConstant: 1),
		])
	}
}
