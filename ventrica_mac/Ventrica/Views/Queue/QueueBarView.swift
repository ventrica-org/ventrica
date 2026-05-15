//
//  VNQueueBarView.swift
//  Ventrica
//

import AppKit
import VentricaKit

// MARK: - VNQueueChipView
private final class QueueChipView: NSView {
	enum ChipStyle {
		case install, dependency, uninstall
	}
	
	private let _label: NSTextField = {
		let v = NSTextField(labelWithString: "")
		v.font = .systemFont(ofSize: 12, weight: .medium)
		return v
	}()
	
	init(name: String, style: ChipStyle) {
		super.init(frame: .zero)
		wantsLayer = true
		layer?.cornerRadius = 6
		layer?.cornerCurve = .continuous
		
		switch style {
		case .install:
			layer?.backgroundColor = NSColor.controlAccentColor.withAlphaComponent(0.15).cgColor
			_label.textColor = .controlAccentColor
		case .dependency:
			layer?.backgroundColor = NSColor.tertiaryLabelColor.withAlphaComponent(0.15).cgColor
			_label.textColor = .secondaryLabelColor
		case .uninstall:
			layer?.backgroundColor = NSColor.systemRed.withAlphaComponent(0.12).cgColor
			_label.textColor = .systemRed
		}
		
		_label.stringValue = name
		_label.translatesAutoresizingMaskIntoConstraints = false
		addSubview(_label)
		
		NSLayoutConstraint.activate([
			_label.topAnchor.constraint(equalTo: topAnchor, constant: 4),
			_label.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -4),
			_label.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 8),
			_label.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -8),
		])
	}
	
	@available(*, unavailable)
	required init?(coder: NSCoder) { fatalError() }
}

// MARK: - PillIconButton

private final class PillIconButton: NSView {
	var action: (() -> Void)?
	
	private let _imageView: NSImageView = {
		let v = NSImageView()
		v.imageScaling = .scaleProportionallyDown
		v.translatesAutoresizingMaskIntoConstraints = false
		return v
	}()
	
	init(background: NSColor, symbolName: String, symbolSize: CGFloat) {
		super.init(frame: .zero)
		wantsLayer = true
		layer?.cornerRadius = 17.5
		layer?.cornerCurve = .continuous
		layer?.masksToBounds = true
		layer?.backgroundColor = background.cgColor
		
		addSubview(_imageView)
		NSLayoutConstraint.activate([
			_imageView.centerXAnchor.constraint(equalTo: centerXAnchor),
			_imageView.centerYAnchor.constraint(equalTo: centerYAnchor),
			_imageView.widthAnchor.constraint(equalToConstant: symbolSize + 4),
			_imageView.heightAnchor.constraint(equalToConstant: symbolSize + 4),
		])
		
		setSymbol(symbolName, size: symbolSize, weight: .bold)
		addGestureRecognizer(NSClickGestureRecognizer(target: self, action: #selector(_clicked)))
	}
	
	@available(*, unavailable)
	required init?(coder: NSCoder) { fatalError() }
	
	func setSymbol(_ name: String, size: CGFloat, weight: NSFont.Weight) {
		let cfg = NSImage.SymbolConfiguration(pointSize: size, weight: weight)
		_imageView.image = NSImage(systemSymbolName: name, accessibilityDescription: nil)?
			.withSymbolConfiguration(cfg)
	}
	
	func setBackground(_ color: NSColor) {
		layer?.backgroundColor = color.cgColor
	}
	
	func setTintColor(_ color: NSColor) {
		_imageView.contentTintColor = color
	}
	
	var isEnabled: Bool = true {
		didSet { alphaValue = isEnabled ? 1 : 0.4 }
	}
	
	@objc private func _clicked() { if isEnabled { action?() } }
}

// MARK: - QueueBarView
final class QueueBarView: NSView {
	private let _blur: NSVisualEffectView = {
		let v = NSVisualEffectView()
		v.material = .hudWindow
		v.blendingMode = .withinWindow
		v.state = .active
		v.wantsLayer = true
		v.layer?.cornerRadius = 28
		v.layer?.cornerCurve = .continuous
		v.layer?.masksToBounds = true
		return v
	}()
	
	private let _chipsScrollView: NSScrollView = {
		let v = NSScrollView()
		v.hasHorizontalScroller = false
		v.hasVerticalScroller = false
		v.drawsBackground = false
		return v
	}()
	
	private let _chipsStack: NSStackView = {
		let v = NSStackView()
		v.orientation = .horizontal
		v.spacing = 5
		v.alignment = .centerY
		return v
	}()
	
	private let _actionButton = PillIconButton(
		background: .controlAccentColor,
		symbolName: "arrow.down",
		symbolSize: 13
	)
	
	private let _clearButton = PillIconButton(
		background: .quaternaryLabelColor,
		symbolName: "xmark",
		symbolSize: 11
	)
	
	override init(frame frameRect: NSRect) {
		super.init(frame: frameRect)
		_setup()
		_observeQueue()
		_refresh()
	}
	
	@available(*, unavailable)
	required init?(coder: NSCoder) { fatalError() }
	
	private func _setup() {
		wantsLayer = true
		layer?.cornerRadius = 28
		layer?.cornerCurve = .continuous
		layer?.borderWidth = 0.75
		layer?.borderColor = NSColor.separatorColor.cgColor
		
		shadow = {
			let s = NSShadow()
			s.shadowBlurRadius = 20
			s.shadowOffset = NSSize(width: 0, height: -4)
			s.shadowColor = NSColor.black.withAlphaComponent(0.25)
			return s
		}()
		
		_blur.translatesAutoresizingMaskIntoConstraints = false
		addSubview(_blur)
		NSLayoutConstraint.activate([
			_blur.topAnchor.constraint(equalTo: topAnchor),
			_blur.bottomAnchor.constraint(equalTo: bottomAnchor),
			_blur.leadingAnchor.constraint(equalTo: leadingAnchor),
			_blur.trailingAnchor.constraint(equalTo: trailingAnchor),
		])
		
		_chipsScrollView.documentView = _chipsStack
		_chipsScrollView.translatesAutoresizingMaskIntoConstraints = false
		_actionButton.translatesAutoresizingMaskIntoConstraints = false
		_clearButton.translatesAutoresizingMaskIntoConstraints = false
		_chipsStack.translatesAutoresizingMaskIntoConstraints = false
		
		_actionButton.setTintColor(.white)
		_clearButton.setTintColor(.secondaryLabelColor)
		
		[_chipsScrollView, _clearButton, _actionButton].forEach { addSubview($0) }
		
		NSLayoutConstraint.activate([
			_actionButton.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -11),
			_actionButton.centerYAnchor.constraint(equalTo: centerYAnchor),
			_actionButton.widthAnchor.constraint(equalToConstant: 35),
			_actionButton.heightAnchor.constraint(equalToConstant: 35),
			
			_clearButton.trailingAnchor.constraint(equalTo: _actionButton.leadingAnchor, constant: -7),
			_clearButton.centerYAnchor.constraint(equalTo: centerYAnchor),
			_clearButton.widthAnchor.constraint(equalToConstant: 35),
			_clearButton.heightAnchor.constraint(equalToConstant: 35),
			
			_chipsScrollView.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 14),
			_chipsScrollView.trailingAnchor.constraint(equalTo: _clearButton.leadingAnchor, constant: -8),
			_chipsScrollView.topAnchor.constraint(equalTo: topAnchor, constant: 8),
			_chipsScrollView.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -8),
		])
		
		_actionButton.action = { [weak self] in self?._applyTapped() }
		_clearButton.action  = { [weak self] in self?._clearTapped() }
	}
	
	private func _observeQueue() {
		NotificationCenter.default.addObserver(
			self,
			selector: #selector(_refresh),
			name: InstallQueue.didChange,
			object: nil
		)
	}
	
	@objc private func _refresh() {
		let queue = InstallQueue.shared
		let installItems = queue.installItems
		let uninstallItems = queue.uninstallItems
		let isApplying = queue.isApplying
		
		_chipsStack.arrangedSubviews.forEach { $0.removeFromSuperview() }
		for item in installItems {
			_chipsStack.addArrangedSubview(QueueChipView(name: item.name, style: item.isDependency ? .dependency : .install))
		}
		for item in uninstallItems {
			_chipsStack.addArrangedSubview(QueueChipView(name: item.name, style: .uninstall))
		}
		
		if isApplying {
			_setActionButton(symbolName: "ellipsis", color: .controlAccentColor)
		} else {
			switch (!installItems.isEmpty, !uninstallItems.isEmpty) {
			case (true, true):  _setActionButton(symbolName: "checkmark", color: .controlAccentColor)
			case (false, true): _setActionButton(symbolName: "trash",     color: .systemRed)
			default:            _setActionButton(symbolName: "arrow.down", color: .controlAccentColor)
			}
		}
		
		let isEmpty = queue.isEmpty
		_actionButton.isEnabled = !isApplying && !isEmpty
		_clearButton.isEnabled  = !isApplying && !isEmpty
	}
	
	private func _setActionButton(symbolName: String, color: NSColor) {
		var resolvedColor = color
		effectiveAppearance.performAsCurrentDrawingAppearance { resolvedColor = color }
		_actionButton.setBackground(resolvedColor)
		_actionButton.setSymbol(symbolName, size: 13, weight: .bold)
	}
	
	@objc private func _applyTapped() {
		InstallQueue.shared.applyAll { [weak self] success, errorMessage in
			if !success, let msg = errorMessage {
				let alert = NSAlert()
				alert.messageText = "Operation Failed"
				alert.informativeText = msg
				alert.alertStyle = .warning
				alert.runModal()
			}
			self?._refresh()
		}
	}
	
	@objc private func _clearTapped() {
		InstallQueue.shared.clear()
	}
	
	override func viewDidChangeEffectiveAppearance() {
		super.viewDidChangeEffectiveAppearance()
		layer?.borderColor = NSColor.separatorColor.cgColor
		_clearButton.setBackground(.quaternaryLabelColor)
		_refresh()
	}
}

final class QueueBarViewController: NSViewController {
	override func loadView() {
		view = QueueBarView()
	}
}
