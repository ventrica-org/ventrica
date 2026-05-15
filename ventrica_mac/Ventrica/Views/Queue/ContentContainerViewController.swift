//
//  VNContentContainerViewController.swift
//  Ventrica
//

import AppKit

final class ContentContainerViewController: NSViewController {
	private var _contentVC: NSViewController
	private let _queueBar = QueueBarViewController()
	private var _queueBarVisible = false
	private var _queueBarObserver: Any?
	private var _barBottomConstraint: NSLayoutConstraint!
	
	init(contentVC: NSViewController) {
		_contentVC = contentVC
		super.init(nibName: nil, bundle: nil)
	}
	
	@available(*, unavailable)
	required init?(coder: NSCoder) { fatalError() }
	
	override func loadView() {
		view = NSView()
	}
	
	override func viewDidLoad() {
		super.viewDidLoad()
		_embedContent(_contentVC)
		_setupQueueBar()
	}
	
	func swapContent(_ newVC: NSViewController) {
		guard newVC !== _contentVC else { return }
		_contentVC.view.removeFromSuperview()
		_contentVC.removeFromParent()
		_contentVC = newVC
		_embedContent(newVC)
	}
	
	private func _embedContent(_ vc: NSViewController) {
		addChild(vc)
		vc.view.translatesAutoresizingMaskIntoConstraints = false
		if _queueBar.parent != nil {
			view.addSubview(vc.view, positioned: .below, relativeTo: _queueBar.view)
		} else {
			view.addSubview(vc.view)
		}
		NSLayoutConstraint.activate([
			vc.view.topAnchor.constraint(equalTo: view.topAnchor),
			vc.view.bottomAnchor.constraint(equalTo: view.bottomAnchor),
			vc.view.leadingAnchor.constraint(equalTo: view.leadingAnchor),
			vc.view.trailingAnchor.constraint(equalTo: view.trailingAnchor),
		])
	}
	
	private func _setupQueueBar() {
		addChild(_queueBar)
		_queueBar.view.translatesAutoresizingMaskIntoConstraints = false
		view.addSubview(_queueBar.view)
		
		_barBottomConstraint = _queueBar.view.bottomAnchor.constraint(
			equalTo: view.bottomAnchor, constant: 76
		)
		
		let widthFraction = _queueBar.view.widthAnchor.constraint(
			equalTo: view.widthAnchor, multiplier: 0.85
		)
		widthFraction.priority = .defaultLow
		
		NSLayoutConstraint.activate([
			_queueBar.view.centerXAnchor.constraint(equalTo: view.centerXAnchor),
			_queueBar.view.widthAnchor.constraint(equalToConstant: 500),
			_queueBar.view.heightAnchor.constraint(equalToConstant: 56),
			_barBottomConstraint,
		])
		
		_queueBarObserver = NotificationCenter.default.addObserver(
			forName: InstallQueue.didChange,
			object: nil,
			queue: .main
		) { [weak self] _ in
			DispatchQueue.main.async {
				self?._syncVisibility(animated: true)
			}
		}
		
		_syncVisibility(animated: false)
	}
	
	private func _syncVisibility(animated: Bool) {
		let shouldShow = !InstallQueue.shared.isEmpty
		guard shouldShow != _queueBarVisible else { return }
		_queueBarVisible = shouldShow
		_barBottomConstraint.constant = shouldShow ? -20 : 76
		
		if animated {
			NSAnimationContext.runAnimationGroup { ctx in
				ctx.duration = 0.35
				ctx.timingFunction = CAMediaTimingFunction(name: .easeInEaseOut)
				ctx.allowsImplicitAnimation = true
				self.view.layoutSubtreeIfNeeded()
			}
		} else {
			view.layoutSubtreeIfNeeded()
		}
	}
}
