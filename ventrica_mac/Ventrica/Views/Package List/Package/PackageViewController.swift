//
//  VNPackageViewController.swift
//  Ventrica
//
//  Created by samsam on 12/30/25.
//

import AppKit
import VentricaUI

private class _TopAlignedStackView: NSStackView {
	override var isFlipped: Bool { true }
}

// MARK: - VNPackageViewController
final class PackageViewController: NSViewController {
	private let _scrollView = NSScrollView()
	private let _stackView = _TopAlignedStackView()
	
	private var _package: Package
	
	init(package: Package) {
		_package = package
		super.init(nibName: nil, bundle: nil)
		self.title = package.name
	}

	@available(*, unavailable)
	required init?(coder: NSCoder) { fatalError() }
	
	override func viewDidLoad() {
		view = NSView()
		_setupScrollView()
		_setupStackView()
		_addHeaderView()
		_seperator()
	}
	
	private func _setupScrollView() {
		_scrollView.translatesAutoresizingMaskIntoConstraints = false
		_scrollView.hasVerticalScroller = true
		_scrollView.drawsBackground = true
		view.addSubview(_scrollView)
		
		NSLayoutConstraint.activate([
			_scrollView.topAnchor.constraint(equalTo: view.topAnchor),
			_scrollView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
			_scrollView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
			_scrollView.trailingAnchor.constraint(equalTo: view.trailingAnchor)
		])
	}
	
	private func _setupStackView() {
		_stackView.orientation = .vertical
		_stackView.alignment = .leading
		_stackView.spacing = 0
		_stackView.translatesAutoresizingMaskIntoConstraints = false
		
		_scrollView.documentView = _stackView
		
		NSLayoutConstraint.activate([
			_stackView.topAnchor.constraint(equalTo: _scrollView.contentView.topAnchor),
			_stackView.leadingAnchor.constraint(equalTo: _scrollView.contentView.leadingAnchor),
			_stackView.trailingAnchor.constraint(equalTo: _scrollView.contentView.trailingAnchor)
		])
	}
	
	private func _addArrangedSubview(_ subview: NSView) {
		_stackView.addArrangedSubview(subview)
		subview.trailingAnchor.constraint(equalTo: _stackView.trailingAnchor).isActive = true
	}
	
	private func _addHeaderView() {
		let v = PackageHeaderView()
		v.configure(package: _package)
		_addArrangedSubview(v)
	}
	
	private func _seperator() {
		let v = VNSeperatorView()
		_addArrangedSubview(v)
	}
}

