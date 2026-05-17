//
//  VNAboutViewController.swift
//  Ventrica
//
//  Created by samsam on 12/30/25.
//

import AppKit

final class AboutViewController: NSViewController {
	private let _iconSize: CGFloat = 95
	
	private let _viewBlur: NSVisualEffectView = {
		let v = NSVisualEffectView()
		v.material = .hudWindow
		v.blendingMode = .behindWindow
		v.state = .active
		return v
	}()
	
	private let _appIconView: NSImageView = {
		let v = NSImageView()
		v.image = NSApp.applicationIconImage
		v.imageScaling = .scaleProportionallyUpOrDown
		return v
	}()
	
	private let _nameLabel: NSTextField = {
		let v = NSTextField(labelWithString: Bundle.main.name)
		v.font = NSFont.boldSystemFont(ofSize: 20)
		v.alignment = .center
		return v
	}()
	
	private let _versionLabel: NSTextField = {
		let v = NSTextField(
			labelWithString: "Version \(Bundle.main.version) (\(Bundle.main.buildVersion))"
		)
		v.font = NSFont.systemFont(ofSize: 13)
		v.textColor = .secondaryLabelColor
		v.alignment = .center
		return v
	}()
	
	private lazy var _contentStack: NSStackView = {
		let v = NSStackView(views: [_appIconView, _nameLabel, _versionLabel])
		v.orientation = .vertical
		v.alignment = .centerX
		v.spacing = 10
		return v
	}()
	
	override func loadView() {
		[_viewBlur, _contentStack].forEach {
			$0.translatesAutoresizingMaskIntoConstraints = false
		}
		
		view = _viewBlur
		view.addSubview(_contentStack)
		
		NSLayoutConstraint.activate([
			_appIconView.widthAnchor.constraint(equalToConstant: _iconSize),
			_appIconView.heightAnchor.constraint(equalToConstant: _iconSize),
			_contentStack.centerXAnchor.constraint(equalTo: view.centerXAnchor),
			_contentStack.centerYAnchor.constraint(equalTo: view.centerYAnchor),
		])
	}
}
