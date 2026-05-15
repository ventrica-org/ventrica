//
//  VNNoPackageViewController.swift
//  Ventrica
//
//  Created by samsam on 3/18/26.
//

import AppKit

final class NoPackageViewController: NSViewController {
	override func loadView() {
		view = NSView()
		
		let icon = NSImageView(image: NSImage(systemSymbolName: "shippingbox", accessibilityDescription: nil)!)
		icon.symbolConfiguration = .init(pointSize: 48, weight: .regular)
		icon.contentTintColor = .tertiaryLabelColor
		icon.translatesAutoresizingMaskIntoConstraints = false
		
		let label = NSTextField(labelWithString: "No Package Selected")
		label.font = .systemFont(ofSize: 17, weight: .medium)
		label.textColor = .secondaryLabelColor
		label.translatesAutoresizingMaskIntoConstraints = false
		
		let subtitle = NSTextField(labelWithString: "Select a package from the list to see its details.")
		subtitle.font = .systemFont(ofSize: 13)
		subtitle.textColor = .tertiaryLabelColor
		subtitle.translatesAutoresizingMaskIntoConstraints = false
		
		view.addSubview(icon)
		view.addSubview(label)
		view.addSubview(subtitle)
		
		NSLayoutConstraint.activate([
			icon.centerXAnchor.constraint(equalTo: view.centerXAnchor),
			icon.centerYAnchor.constraint(equalTo: view.centerYAnchor, constant: -32),
			label.topAnchor.constraint(equalTo: icon.bottomAnchor, constant: 16),
			label.centerXAnchor.constraint(equalTo: view.centerXAnchor),
			subtitle.topAnchor.constraint(equalTo: label.bottomAnchor, constant: 6),
			subtitle.centerXAnchor.constraint(equalTo: view.centerXAnchor)
		])
	}
}
