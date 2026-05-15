//
//  VNPackageHeaderView.swift
//  VentricaUI
//
//  Created by samsam on 3/16/26.
//

import AppKit
import SwiftUI
import VentricaUI

final class PackageHeaderView: NSView {
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
	private var _queueObserver: Any?
	
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
	
	func configure(package: Package) {
		_nameLabel.stringValue = package.name
		_descriptionLabel.stringValue = package.description
		_iconView.image = VNCategoryIdentifier(package.category).sectionIcon.image()
		
		if let iconString = package.icon, let url = URL(string: iconString) {
			Task { [weak self] in
				guard let self, let image = await ImageLoader.shared.load(url: url) else { return }
				await MainActor.run { self._iconView.image = image }
			}
		}

		_syncButtonState(for: package)

		if let obs = _queueObserver { NotificationCenter.default.removeObserver(obs) }
		let packageName = package.name
		_queueObserver = NotificationCenter.default.addObserver(
			forName: InstallQueue.didChange,
			object: nil,
			queue: .main
		) { [weak self] _ in
			guard let self else { return }
			DispatchQueue.main.async {
				self._syncButtonStateForName(packageName)
			}
		}

		_getButton.onTap = { [weak self] in
			guard let self else { return }
			switch self._getButton.buttonState {
			case .installed:
				InstallQueue.shared.enqueueUninstall(package)
			case .get:
				InstallQueue.shared.enqueue(package)
			default:
				break
			}
		}
	}

	private func _syncButtonState(for package: Package) {
		_syncButtonStateForName(package.name)
	}

	private func _syncButtonStateForName(_ name: String) {
		let queue = InstallQueue.shared
		if queue.isQueuedForUninstall(name) {
			_getButton.buttonState = .uninstallQueued
		} else if queue.isInstalled(name) {
			_getButton.buttonState = .installed
		} else if queue.isQueued(name) {
			_getButton.buttonState = .queued
		} else {
			_getButton.buttonState = .get
		}
	}
	
	fileprivate func configure() {
		_nameLabel.stringValue =  "uikittools-ng"
		_descriptionLabel.stringValue = "Next-gen uikittools for iOS 11+ (though probably will work on 9+)"
		_iconView.image = VNCategoryIdentifier("developer").sectionIcon.image()
	}
}

#Preview(PackageHeaderView.className()) {
	struct Preview: NSViewRepresentable {
		func makeNSView(context: Context) -> NSView {
			let cell = PackageHeaderView()
			cell.configure()
			return cell
		}
		
		func updateNSView(_ nsView: NSView, context: Context) {}
	}
	
	return Preview()
}

final class VNPillButton: NSButton {
	enum State {
		case get
		case queued
		case installed
		case uninstallQueued
	}

	var buttonState: State = .get {
		didSet { _refresh() }
	}

	var onTap: (() -> Void)?

	override init(frame frameRect: NSRect) {
		super.init(frame: frameRect)
		_configure()
	}

	@available(*, unavailable)
	required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }

	private func _configure() {
		isBordered = false
		wantsLayer = true
		layer?.cornerCurve = .continuous
		target = self
		action = #selector(_buttonTapped)
		_refresh()
	}

	private func _refresh() {
		effectiveAppearance.performAsCurrentDrawingAppearance {
			switch self.buttonState {
			case .get:
				self.layer?.backgroundColor = NSColor.controlAccentColor.cgColor
				self.isEnabled = true
				self.attributedTitle = NSAttributedString(
					string: "Get",
					attributes: [
						.font: NSFont.systemFont(ofSize: 13, weight: .semibold),
						.foregroundColor: NSColor.white
					]
				)
			case .queued:
				self.layer?.backgroundColor = NSColor.systemGreen.cgColor
				self.isEnabled = false
				self.attributedTitle = NSAttributedString(
					string: "Queued",
					attributes: [
						.font: NSFont.systemFont(ofSize: 13, weight: .semibold),
						.foregroundColor: NSColor.white
					]
				)
			case .installed:
				self.layer?.backgroundColor = NSColor.systemRed.withAlphaComponent(0.12).cgColor
				self.isEnabled = true
				self.attributedTitle = NSAttributedString(
					string: "Uninstall",
					attributes: [
						.font: NSFont.systemFont(ofSize: 13, weight: .semibold),
						.foregroundColor: NSColor.systemRed
					]
				)
			case .uninstallQueued:
				self.layer?.backgroundColor = NSColor.systemRed.withAlphaComponent(0.07).cgColor
				self.isEnabled = false
				self.attributedTitle = NSAttributedString(
					string: "Queued",
					attributes: [
						.font: NSFont.systemFont(ofSize: 13, weight: .semibold),
						.foregroundColor: NSColor.systemRed.withAlphaComponent(0.5)
					]
				)
			}
		}
	}

	@objc private func _buttonTapped() {
		guard buttonState == .get || buttonState == .installed else { return }
		onTap?()
	}

	override var intrinsicContentSize: NSSize { NSSize(width: 90, height: 30) }

	override func layout() {
		super.layout()
		layer?.cornerRadius = bounds.height / 2
	}

	override func viewDidChangeEffectiveAppearance() {
		super.viewDidChangeEffectiveAppearance()
		_refresh()
	}

	override func mouseDown(with event: NSEvent) {
		guard buttonState == .get || buttonState == .installed else { return }
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
