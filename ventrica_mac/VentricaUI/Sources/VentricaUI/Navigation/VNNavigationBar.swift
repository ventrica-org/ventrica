//
//  VNNavigationBar.swift
//  VentricaUI
//
//  Created by samsam on 12/26/25.
//

import AppKit

public final class VNNavigationBar: NSView {

	internal let visualEffectView = NSVisualEffectView()
	internal let contentContainer = NSView()
	internal let titleLabel = NSTextField(labelWithString: "")
	internal let rightStack = NSStackView()
	internal let bottomBorder = NSBox()
	internal let backButton = VNButton()

	private var backButtonLeadingConstraint: NSLayoutConstraint!

	/// When `true`, the title label is not dimmed during fade-out.
	public var keepsTitleVisible: Bool = false

	public weak var delegate: VNNavigationBarDelegate?

	public override init(frame frameRect: NSRect) {
		super.init(frame: frameRect)
		setup()
	}

	@available(*, unavailable)
	required init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}

	// MARK: - Public API

	public func setTitle(_ title: String) {
		titleLabel.stringValue = title
	}

	public func setRightButtons(_ buttons: [NSControl]) {
		rightStack.arrangedSubviews.forEach { $0.removeFromSuperview() }
		buttons.forEach {
			$0.setSemanticStyle(.titlebar)
			$0.controlSize = .large
			rightStack.addArrangedSubview($0)
		}
	}

	public func setShowsBackButton(_ shows: Bool, animated: Bool = true) {
		let changes = {
			self.backButton.alphaValue = shows ? 1 : 0
		}

		animated
		 ? NSAnimationContext.runAnimationGroup { ctx in ctx.duration = 0.2; changes() }
		 : changes()
	}

	public func setFullCoverStyle(_ isFullCover: Bool) {
		backButtonLeadingConstraint.constant = isFullCover ? 92 : 12
	}

	public func setBackgroundMaterial(_ material: NSVisualEffectView.Material) {
		visualEffectView.material = material
	}

	public func setBottomBorderVisible(_ visible: Bool) {
		bottomBorder.isHidden = !visible
	}

	public func setContentAlpha(_ alpha: CGFloat, animated: Bool = true) {
		let changes = {
			self.contentContainer.alphaValue = alpha
		}

		animated
		 ? NSAnimationContext.runAnimationGroup { ctx in ctx.duration = 0.2; changes() }
		 : changes()
	}

	/// Intentional customization point
	public var contentView: NSView {
		contentContainer
	}

	// MARK: - Setup

	private func setup() {
		wantsLayer = true
		translatesAutoresizingMaskIntoConstraints = false

		visualEffectView.material = .headerView
		visualEffectView.blendingMode = .withinWindow
		visualEffectView.state = .active
		visualEffectView.translatesAutoresizingMaskIntoConstraints = false
		addSubview(visualEffectView)

		contentContainer.translatesAutoresizingMaskIntoConstraints = false
		addSubview(contentContainer)

		if let image = NSImage(
			systemSymbolName: "chevron.left",
			accessibilityDescription: "Back"
		) {
			backButton.image = image
			backButton.imagePosition = .imageOnly
		}

		backButton.target = self
		backButton.action = #selector(backTapped)
		backButton.translatesAutoresizingMaskIntoConstraints = false
		backButton.setSemanticStyle(.titlebar)
		backButton.controlSize = .large
		addSubview(backButton)

		titleLabel.font = .systemFont(ofSize: 14, weight: .semibold)
		titleLabel.textColor = .secondaryLabelColor
		titleLabel.translatesAutoresizingMaskIntoConstraints = false
		contentContainer.addSubview(titleLabel)

		rightStack.orientation = .horizontal
		rightStack.spacing = 8
		rightStack.translatesAutoresizingMaskIntoConstraints = false
		contentContainer.addSubview(rightStack)

		bottomBorder.boxType = .custom
		bottomBorder.borderColor = NSColor.gray.withAlphaComponent(0.5)
		bottomBorder.borderWidth = 1
		bottomBorder.translatesAutoresizingMaskIntoConstraints = false
		addSubview(bottomBorder)

		backButtonLeadingConstraint =
			backButton.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 12)

		NSLayoutConstraint.activate([
			visualEffectView.topAnchor.constraint(equalTo: topAnchor),
			visualEffectView.leadingAnchor.constraint(equalTo: leadingAnchor),
			visualEffectView.trailingAnchor.constraint(equalTo: trailingAnchor),
			visualEffectView.bottomAnchor.constraint(equalTo: bottomAnchor),

			contentContainer.topAnchor.constraint(equalTo: topAnchor),
			contentContainer.leadingAnchor.constraint(equalTo: leadingAnchor),
			contentContainer.trailingAnchor.constraint(equalTo: trailingAnchor),
			contentContainer.bottomAnchor.constraint(equalTo: bottomAnchor),

			backButtonLeadingConstraint,
			backButton.centerYAnchor.constraint(equalTo: centerYAnchor),
			backButton.widthAnchor.constraint(equalToConstant: 32),

			titleLabel.centerXAnchor.constraint(equalTo: centerXAnchor),
			titleLabel.centerYAnchor.constraint(equalTo: centerYAnchor),

			rightStack.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -12),
			rightStack.centerYAnchor.constraint(equalTo: centerYAnchor),

			bottomBorder.heightAnchor.constraint(equalToConstant: 1),
			bottomBorder.leadingAnchor.constraint(equalTo: leadingAnchor),
			bottomBorder.trailingAnchor.constraint(equalTo: trailingAnchor),
			bottomBorder.bottomAnchor.constraint(equalTo: bottomAnchor),

			heightAnchor.constraint(equalToConstant: 52)
		])
	}

	// MARK: - Actions

	@objc private func backTapped() {
		delegate?.navigationBarDidTapBack(self)
	}
	
	func updateHistoryMenu(stackTitles: [String], fullCoverTitles: [String]) {
		let menu = NSMenu()
		
		for (index, title) in fullCoverTitles.enumerated().reversed() {
			let item = NSMenuItem(title: title, action: #selector(menuItemTapped(_:)), keyEquivalent: "")
			item.target = self
			item.representedObject = ["index": index, "isFullCover": true]
			menu.addItem(item)
		}
		
		if !fullCoverTitles.isEmpty && !stackTitles.isEmpty { menu.addItem(.separator()) }
		
		for (index, title) in stackTitles.enumerated().reversed() {
			let item = NSMenuItem(title: title, action: #selector(menuItemTapped(_:)), keyEquivalent: "")
			item.target = self
			item.representedObject = ["index": index, "isFullCover": false]
			menu.addItem(item)
		}
		
		backButton.menu = menu.items.isEmpty
		 ? nil
		 : menu
	}
	
	@objc private func menuItemTapped(_ sender: NSMenuItem) {
		guard
			let data = sender.representedObject as? [String: Any],
			let index = data["index"] as? Int,
			let isFullCover = data["isFullCover"] as? Bool else
		{
			return
		}
		
		delegate?.navigationBar(self, didSelectBackAt: index, isFullCover: isFullCover)
	}

	/// Applies an alpha to the bar's components, respecting `keepsTitleVisible` and `persistentButtons`.
	/// Call this inside an `NSAnimationContext` block to animate.
	internal func applyFadeAlpha(_ alpha: CGFloat) {
		visualEffectView.animator().alphaValue = alpha
		bottomBorder.animator().alphaValue = alpha
		if !keepsTitleVisible {
			titleLabel.animator().alphaValue = alpha
		}
		for view in rightStack.arrangedSubviews {
			view.animator().alphaValue = 1
		}
	}
}


public final class VNButton: NSButton {
	public override func mouseDown(with event: NSEvent) {
		super.mouseDown(with: event)
	}
	
	public override func rightMouseDown(with event: NSEvent) {
		if let menu = self.menu {
			NSMenu.popUpContextMenu(menu, with: event, for: self)
		} else {
			super.rightMouseDown(with: event)
		}
	}
}

public protocol VNNavigationBarDelegate: AnyObject {
	func navigationBarDidTapBack(_ navigationBar: VNNavigationBar)
	func navigationBar(_ navigationBar: VNNavigationBar, didSelectBackAt index: Int, isFullCover: Bool)
}
