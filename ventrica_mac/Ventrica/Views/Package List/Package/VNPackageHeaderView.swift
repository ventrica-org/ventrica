//
//  VNPackageHeaderView.swift
//  VentricaUI
//
//  Created by samsam on 3/16/26.
//

import AppKit
import SwiftUI
import VentricaUI

final class VNPackageHeaderView: NSView {
	private let _padding: CGFloat = 32
	
	private let _iconView: NSImageView = {
		let v = NSImageView()
		v.imageScaling = .scaleProportionallyUpOrDown
		v.wantsLayer = true
		v.layer?.cornerRadius = 27.5766
		v.layer?.cornerCurve = .continuous
		v.layer?.masksToBounds = true
		v.layer?.borderWidth = 1
		v.layer?.borderColor = NSColor.gray.withAlphaComponent(0.3).cgColor
		return v
	}()
	
	private let _nameLabel: NSTextField = {
		let v = NSTextField(labelWithString: "")
		v.font = .systemFont(ofSize: 25, weight: .bold)
		v.lineBreakMode = .byTruncatingTail
		v.maximumNumberOfLines = 1
		return v
	}()
	
	private let _descriptionLabel: NSTextField = {
		let v = NSTextField(labelWithString: "")
		v.font = .systemFont(ofSize: 15)
		v.textColor = .secondaryLabelColor
		v.lineBreakMode = .byWordWrapping
		v.maximumNumberOfLines = 2
		return v
	}()
	
	private let _spacer: NSView = {
		let v = NSView()
		v.setContentHuggingPriority(.defaultLow, for: .vertical)
		v.setContentCompressionResistancePriority(.defaultLow, for: .vertical)
		return v
	}()
	
	private let _getButton = VNPillButton()
	
	private let _textStack: NSStackView = {
		let v = NSStackView()
		v.orientation = .vertical
		v.alignment = .leading
		v.spacing = 4
		return v
	}()
	
	override init(frame frameRect: NSRect) {
		super.init(frame: frameRect)
		_setup()
	}
	
	@available(*, unavailable)
	required public init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}
	
	private func _setup() {
		[_nameLabel, _descriptionLabel, _spacer, _getButton].forEach {
			_textStack.addArrangedSubview($0)
		}
		[_iconView, _textStack].forEach {
			addSubview($0)
			$0.translatesAutoresizingMaskIntoConstraints = false
		}
		
		// so it hugs if theres no space for it
		_nameLabel.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
		_descriptionLabel.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
		
		NSLayoutConstraint.activate([
			heightAnchor.constraint(equalToConstant: 150),
			_iconView.topAnchor.constraint(equalTo: topAnchor, constant: _padding / 2),
			_iconView.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -_padding / 2),
			_iconView.widthAnchor.constraint(equalTo: _iconView.heightAnchor),
			_iconView.leadingAnchor.constraint(equalTo: leadingAnchor, constant: _padding),
			
			_textStack.topAnchor.constraint(equalTo: topAnchor, constant: _padding / 2),
			_textStack.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -_padding / 2),
			_textStack.leadingAnchor.constraint(equalTo: _iconView.trailingAnchor, constant: _padding),
			_textStack.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -_padding)
		])
	}
	
	func configure(package: VNPackage) {
		_nameLabel.stringValue = package.name
		_descriptionLabel.stringValue = package.description
		_iconView.image = VNCategoryIdentifier(package.category).sectionIcon.image()
		
		if let iconString = package.icon, let url = URL(string: iconString) {
			Task { [weak self] in
				guard let self, let image = await VNImageLoader.shared.load(url: url) else { return }
				await MainActor.run { self._iconView.image = image }
			}
		}
	}
	
	fileprivate func configure() {
		_nameLabel.stringValue =  "uikittools-ng"
		_descriptionLabel.stringValue = "Next-gen uikittools for iOS 11+ (though probably will work on 9+)"
		_iconView.image = VNCategoryIdentifier("developer").sectionIcon.image()
	}
}

#Preview(VNPackageHeaderView.className()) {
	struct Preview: NSViewRepresentable {
		func makeNSView(context: Context) -> NSView {
			let cell = VNPackageHeaderView()
			cell.configure()
			return cell
		}
		
		func updateNSView(_ nsView: NSView, context: Context) {}
	}
	
	return Preview()
}

#warning("aa")
final class VNPillButton: NSButton {
	override init(frame frameRect: NSRect) {
		super.init(frame: frameRect)
		title = "Get"
		_configure()
	}
	
	@available(*, unavailable)
	required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }
	
	private func _configure() {
		isBordered = false
		wantsLayer = true
		layer?.cornerCurve = .continuous
		_refresh()
	}
	
	private func _refresh() {
		var accent = NSColor.controlAccentColor
		effectiveAppearance.performAsCurrentDrawingAppearance { accent = NSColor.controlAccentColor }
		layer?.backgroundColor = accent.cgColor
		attributedTitle = NSAttributedString(
			string: title,
			attributes: [
				.font: NSFont.systemFont(ofSize: 13, weight: .semibold),
				.foregroundColor: NSColor.white
			]
		)
	}
	
	override var intrinsicContentSize: NSSize { NSSize(width: 80, height: 30) }
	
	override func layout() {
		super.layout()
		layer?.cornerRadius = bounds.height / 2
	}
	
	override func viewDidChangeEffectiveAppearance() {
		super.viewDidChangeEffectiveAppearance()
		_refresh()
	}
	
	override func mouseDown(with event: NSEvent) {
		NSAnimationContext.runAnimationGroup { ctx in
			ctx.duration = 0.08
			self.layer?.opacity = 0.6
		}
		super.mouseDown(with: event)
		NSAnimationContext.runAnimationGroup { ctx in
			ctx.duration = 0.2
			self.layer?.opacity = 1
		}
	}
}
