//
//  VNViewController.swift
//  Ventrica
//
//  Created by samsam on 12/24/25.
//

import AppKit

open class VNViewController: NSViewController, @MainActor VNNavigable {
	let navBar = VNNavigationBar()
	private let hoverZone = NSView()
	private var navBarVisible = false
	nonisolated(unsafe) private var scrollNotificationToken: NSObjectProtocol?
	
	/// Scroll distance (in points) past which the nav bar becomes visible.
	public var navBarFadeScrollOffset: CGFloat = -52
	
	/// When `false` the bar is always fully visible and never fades out.
	public var navBarFadeEnabled: Bool = true {
		didSet {
			guard !navBarFadeEnabled else { return }
			navBarVisible = true
			NSAnimationContext.runAnimationGroup { ctx in
				ctx.duration = 0.2
				self.navBar.applyFadeAlpha(1)
			}
			navBar.backButton.alphaValue = 1
		}
	}
	
	public var navBarTitle: String? { titleText }
	public var navBarRightButtons: [NSControl]? { navBarButtons }
	public var navigationBar: VNNavigationBar? { navBar }
	public var navBarButtons: [NSControl] = [] {
		didSet {
			navBar.setRightButtons(navBarButtons)
		}
	}
	public var titleText: String = "" {
		didSet {
			navBar.setTitle(titleText)
		}
	}
	
	public var navigationController: VNNavigationController? {
		var parentVC = parent
		while parentVC != nil {
			if let nav = parentVC as? VNNavigationController {
				return nav
			}
			parentVC = parentVC?.parent
		}
		return nil
	}
	
	required public init(titleText: String) {
		super.init(nibName: nil, bundle: nil)
		self.titleText = titleText
	}
	
	@available(*, unavailable)
	required public init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}
	
	open override func loadView() {
		view = NSView()
		
		// show content by default
		navBar.translatesAutoresizingMaskIntoConstraints = false
		navBar.visualEffectView.alphaValue = 1
		navBar.bottomBorder.alphaValue = 1
		view.addSubview(navBar, positioned: .above, relativeTo: nil)
		
		NSLayoutConstraint.activate([
			navBar.topAnchor.constraint(equalTo: view.topAnchor),
			navBar.leadingAnchor.constraint(equalTo: view.leadingAnchor),
			navBar.trailingAnchor.constraint(equalTo: view.trailingAnchor),
			navBar.heightAnchor.constraint(equalToConstant: 52)
		])
		
		setupHoverZone()
	}
	
	open override func viewDidLoad() {
		super.viewDidLoad()
		view.addSubview(navBar, positioned: .above, relativeTo: nil)
		view.addSubview(hoverZone, positioned: .above, relativeTo: nil)
		updateNavBarVisibility(show: true)
	}
	
	private func setupHoverZone() {
		hoverZone.wantsLayer = true
		hoverZone.layer?.backgroundColor = NSColor.clear.cgColor
		hoverZone.translatesAutoresizingMaskIntoConstraints = false
		view.addSubview(hoverZone, positioned: .above, relativeTo: nil)
		NSLayoutConstraint.activate([
			hoverZone.topAnchor.constraint(equalTo: view.topAnchor),
			hoverZone.leadingAnchor.constraint(equalTo: view.leadingAnchor),
			hoverZone.trailingAnchor.constraint(equalTo: view.trailingAnchor),
			hoverZone.heightAnchor.constraint(equalToConstant: 40)
		])
		let trackingArea = NSTrackingArea(
			rect: .zero,
			options: [.mouseEnteredAndExited, .activeInKeyWindow, .inVisibleRect],
			owner: self,
			userInfo: nil
		)
		hoverZone.addTrackingArea(trackingArea)
	}
	
	open override func mouseEntered(with event: NSEvent) {
		updateNavBarVisibility(show: true)
	}
	
	open override func mouseExited(with event: NSEvent) {
		let offset = (view.subviews.compactMap { $0 as? NSScrollView }.first?.contentView.bounds.origin.y) ?? 0
		if offset <= navBarFadeScrollOffset {
			updateNavBarVisibility(show: false)
		}
	}

	public func observeScrollView(_ scrollView: NSScrollView) {
		if let token = scrollNotificationToken {
			NotificationCenter.default.removeObserver(token)
		}
		scrollView.contentView.postsBoundsChangedNotifications = true
		scrollNotificationToken = NotificationCenter.default.addObserver(
			forName: NSView.boundsDidChangeNotification,
			object: scrollView.contentView,
			queue: .main
		) { [weak self, weak scrollView] _ in
			guard let self, let scrollView else { return }
			DispatchQueue.main.async {
				self.scrollViewDidScroll(scrollView)
			}
		}
	}
	
	deinit {
		if let token = scrollNotificationToken {
			NotificationCenter.default.removeObserver(token)
		}
	}
	
	func updateNavBarVisibility(show: Bool) {
		guard navBarFadeEnabled else { return }
		guard show != navBarVisible else { return }
		navBarVisible = show
		NSAnimationContext.runAnimationGroup { context in
			context.duration = 0.25
			self.navBar.applyFadeAlpha(show ? 1 : 0)
		}
		navBar.backButton.alphaValue = 1
	}
	
	public func scrollViewDidScroll(_ scrollView: NSScrollView) {
		let offset = scrollView.contentView.bounds.origin.y
		updateNavBarVisibility(show: offset > navBarFadeScrollOffset)
	}
}
