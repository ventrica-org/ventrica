//
//  VNNoPackageViewController.swift
//  Ventrica
//
//  Created by samsam on 3/18/26.
//

import AppKit

final class NoPackageViewController: NSViewController {
	private let _shippingImageView: NSImageView = {
		let v = NSImageView(image: NSImage(
			systemSymbolName: "shippingbox",
			accessibilityDescription: nil
		)!)
		v.symbolConfiguration = .init(pointSize: 48, weight: .regular)
		v.contentTintColor = .tertiaryLabelColor
		return v
	}()
	
	private let _titleLabel: NSTextField = {
		let v = NSTextField(labelWithString: "No Package Selected")
		v.font = .systemFont(ofSize: 17, weight: .medium)
		v.textColor = .secondaryLabelColor
		return v
	}()
	
	private let _subtitleLabel: NSTextField = {
		let v = NSTextField(labelWithString: "Select a package from the list to see its details.")
		v.font = .systemFont(ofSize: 13)
		v.textColor = .tertiaryLabelColor
		return v
	}()
	
	private lazy var _contentStack: NSStackView = {
		let v = NSStackView(views: [_shippingImageView, _titleLabel, _subtitleLabel])
		v.orientation = .vertical
		v.alignment = .centerX
		v.spacing = 13
		v.translatesAutoresizingMaskIntoConstraints = false
		return v
	}()
	
	override func loadView() {
		view = .init()
		view.addSubview(_contentStack)
		
		NSLayoutConstraint.activate([
			_contentStack.centerXAnchor.constraint(equalTo: view.centerXAnchor),
			_contentStack.centerYAnchor.constraint(equalTo: view.centerYAnchor),
		])
	}
}
