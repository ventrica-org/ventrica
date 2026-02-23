//
//  VNNavigationController.swift
//  VentricaUI
//
//  Created by samsam on 12/26/25.
//

import AppKit

// MARK: - Protocols
public protocol VNNavigable: AnyObject {
	var navBarTitle: String? { get }
	var navBarRightButtons: [NSControl]? { get }
	var navigationBar: VNNavigationBar? { get }
}

// MARK: - Navigation Controller

open class VNNavigationController: NSViewController, @MainActor VNNavigationBarDelegate {
	private var stack: [NSViewController] = []
	private var fullCoverVCs: [NSViewController] = []
	private var fullCoverOverlays: [NSViewController: NSView] = [:]
	
	private let container = NSView()
	private var isTransitioning = false
	
	public init(rootViewController: NSViewController) {
		super.init(nibName: nil, bundle: nil)
		self.pushViewController(rootViewController, animated: false)
	}
	
	@available(*, unavailable)
	required public init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}
	
	public override func loadView() {
		view = NSView()
		container.translatesAutoresizingMaskIntoConstraints = false
		view.addSubview(container)
		NSLayoutConstraint.activate([
			container.topAnchor.constraint(equalTo: view.topAnchor),
			container.leadingAnchor.constraint(equalTo: view.leadingAnchor),
			container.trailingAnchor.constraint(equalTo: view.trailingAnchor),
			container.bottomAnchor.constraint(equalTo: view.bottomAnchor)
		])
	}
	
	public func pushViewController(_ vc: NSViewController, animated: Bool = true) {
		guard !isTransitioning else { return }
		if !fullCoverVCs.isEmpty { pushFullCoverViewController(vc, animated: animated); return }
		
		guard let current = stack.last else {
			addChild(vc); stack.append(vc); addContent(vc); wireNavBar(for: vc); return
		}
		
		addChild(vc); stack.append(vc)
		performTransition(from: current, to: vc, animated: animated, popping: false)
	}
	
	public func popViewController(animated: Bool = true) {
		guard !isTransitioning else { return }
		if !fullCoverVCs.isEmpty { popFullCoverViewController(animated: animated); return }
		guard stack.count > 1 else { return }
		
		let top = stack.removeLast()
		let newTop = stack.last!
		performTransition(from: top, to: newTop, animated: animated, popping: true) {
			top.removeFromParent()
		}
	}
	
	// MARK: - Full Cover Logic
	public func pushFullCoverViewController(_ vc: NSViewController, animated: Bool = true) {
		guard let window = view.window, !isTransitioning else { return }
		isTransitioning = true
		setNavBarInteractionEnabled(false)
		
		let isFirstFullCover = fullCoverVCs.isEmpty
		let previousVC = isFirstFullCover ? nil : fullCoverVCs.last
		let hostView = window.contentView!
		let width = hostView.bounds.width
		
		addChild(vc)
		fullCoverVCs.append(vc)
		
		let fullView = vc.view
		fullView.wantsLayer = true
		fullView.frame = hostView.bounds
		fullView.frame.origin.x = width
		
		let transitionDimmer = createDimmingOverlay(frame: hostView.bounds)
		transitionDimmer.alphaValue = 0.0
		
		if isFirstFullCover {
			hostView.addSubview(transitionDimmer, positioned: .above, relativeTo: container)
		} else if let prev = previousVC {
			hostView.addSubview(transitionDimmer, positioned: .above, relativeTo: prev.view)
		}
		
		hostView.addSubview(fullView, positioned: .above, relativeTo: transitionDimmer)
		fullCoverOverlays[vc] = transitionDimmer
		
		NSAnimationContext.runAnimationGroup({ ctx in
			ctx.duration = animated ? 0.35 : 0
			ctx.timingFunction = .init(name: .easeInEaseOut)
			fullView.animator().frame.origin.x = 0
			if isFirstFullCover {
				container.animator().alphaValue = 0.6
				transitionDimmer.animator().alphaValue = 0.3
			} else {
				previousVC?.view.animator().frame.origin.x = -width * 0.3
				transitionDimmer.animator().alphaValue = 0.4
			}
		}, completionHandler: {
			Task { @MainActor in
				self.activateFullCoverConstraints(for: fullView, in: hostView)
				self.isTransitioning = false
				self.setNavBarInteractionEnabled(true)
				self.wireNavBar(for: vc)
			}
		})
	}
	
	public func popFullCoverViewController(animated: Bool = true) {
		guard !fullCoverVCs.isEmpty, let fullVC = fullCoverVCs.last, !isTransitioning else { return }
		isTransitioning = true
		setNavBarInteractionEnabled(false)
		
		let fullView = fullVC.view
		let width = fullView.frame.width
		let isLastFullCover = fullCoverVCs.count == 1
		let previousVC = isLastFullCover ? nil : fullCoverVCs[fullCoverVCs.count - 2]
		let currentDimmer = fullCoverOverlays[fullVC]
		
		removeFullCoverConstraints(for: fullView)
		fullView.translatesAutoresizingMaskIntoConstraints = true
		
		let animView: NSView
		if let snap = snapshotView(of: fullView) {
			snap.frame = fullView.frame
			fullView.superview?.addSubview(snap, positioned: .above, relativeTo: fullView)
			fullView.removeFromSuperview()
			animView = snap
		} else {
			animView = fullView
		}
		
		NSAnimationContext.runAnimationGroup({ ctx in
			ctx.duration = animated ? 0.3 : 0
			animView.animator().frame.origin.x = width
			currentDimmer?.animator().alphaValue = 0
			if isLastFullCover { container.animator().alphaValue = 1.0 }
			else { previousVC?.view.animator().frame.origin.x = 0 }
		}, completionHandler: {
			Task { @MainActor in
				animView.removeFromSuperview()
				currentDimmer?.removeFromSuperview()
				fullVC.removeFromParent()
				self.fullCoverOverlays.removeValue(forKey: fullVC)
				self.fullCoverVCs.removeLast()
				
				let currentTop = self.fullCoverVCs.last ?? self.stack.last!
				if let lastVC = self.fullCoverVCs.last { self.activateFullCoverConstraints(for: lastVC.view, in: self.view.window?.contentView) }
				
				self.isTransitioning = false
				self.setNavBarInteractionEnabled(true)
				self.wireNavBar(for: currentTop)
			}
		})
	}
	
	public func navigationBarDidTapBack(_ navigationBar: VNNavigationBar) {
		popViewController(animated: true)
	}
	
	public func navigationBar(_ navigationBar: VNNavigationBar, didSelectBackAt index: Int, isFullCover: Bool) {
		guard !isTransitioning else { return }
		
		if isFullCover {
			let controllersToRemove = Array(fullCoverVCs[(index + 1)...])
			
			controllersToRemove.forEach { vc in
				vc.view.removeFromSuperview()
				fullCoverOverlays[vc]?.removeFromSuperview()
				fullCoverOverlays.removeValue(forKey: vc)
				vc.removeFromParent()
			}
			
			fullCoverVCs = Array(fullCoverVCs[...index])
			
			popFullCoverViewController(animated: true)
			
		} else {
			fullCoverVCs.forEach { vc in
				vc.view.removeFromSuperview()
				fullCoverOverlays[vc]?.removeFromSuperview()
				vc.removeFromParent()
			}
			fullCoverVCs.removeAll()
			fullCoverOverlays.removeAll()
			container.alphaValue = 1.0
			
			guard let currentTop = stack.last else { return }

			let intermediateControllers = Array(stack[(index + 1)..<(stack.count - 1)])
			intermediateControllers.forEach { $0.removeFromParent() }

			let preservedHistory = Array(stack[0...index])
			stack = preservedHistory + [currentTop]

			popViewController(animated: true)
		}
	}
	
	private func performTransition(
		from: NSViewController,
		to: NSViewController,
		animated: Bool,
		popping: Bool,
		completion: (@MainActor () -> Void)? = nil
	) {
		isTransitioning = true
		setNavBarInteractionEnabled(false)
		
		let width = container.bounds.width
		let dimmer = createDimmingOverlay(frame: container.bounds)
		dimmer.alphaValue = popping ? 0.4 : 0.0
		
		[from.view, to.view].forEach {
			$0.wantsLayer = true
			removeConstraints(for: $0, in: container)
			$0.translatesAutoresizingMaskIntoConstraints = true
			$0.frame = container.bounds
		}
		
		let fromAnimView: NSView
		if let snap = snapshotView(of: from.view) {
			snap.frame = container.bounds
			container.addSubview(snap, positioned: .above, relativeTo: from.view)
			from.view.removeFromSuperview()
			fromAnimView = snap
		} else {
			fromAnimView = from.view
		}
		
		if popping {
			to.view.frame.origin.x = -width
			container.addSubview(to.view, positioned: .below, relativeTo: fromAnimView)
			container.addSubview(dimmer, positioned: .above, relativeTo: to.view)
		} else {
			to.view.frame.origin.x = width
			container.addSubview(dimmer, positioned: .above, relativeTo: fromAnimView)
			container.addSubview(to.view, positioned: .above, relativeTo: dimmer)
		}
		
		NSAnimationContext.runAnimationGroup({ ctx in
			ctx.duration = animated ? 0.35 : 0
			ctx.timingFunction = .init(name: .easeInEaseOut)
			fromAnimView.animator().frame.origin.x = popping ? width : -width * 0.3
			to.view.animator().frame.origin.x = 0
			dimmer.animator().alphaValue = popping ? 0 : 0.4
		}, completionHandler: {
			Task { @MainActor in
				fromAnimView.removeFromSuperview()
				dimmer.removeFromSuperview()
				self.activateConstraints(for: to.view)
				self.isTransitioning = false
				self.setNavBarInteractionEnabled(true)
				self.wireNavBar(for: to)
				completion?()
			}
		})
	}
	
	private func wireNavBar(for vc: NSViewController) {
		guard let nav = vc as? VNNavigable, let navBar = nav.navigationBar else { return }
		navBar.delegate = self
		navBar.setFullCoverStyle(!fullCoverVCs.isEmpty)
		navBar.backButton.isHidden = (stack.count <= 1 && fullCoverVCs.isEmpty)
		navBar.setTitle(nav.navBarTitle ?? "")
		navBar.setRightButtons(nav.navBarRightButtons ?? [])
		
		let stackTitles = stack.dropLast().compactMap { ($0 as? VNNavigable)?.navBarTitle ?? "Back" }
		let fullTitles = fullCoverVCs.dropLast().compactMap { ($0 as? VNNavigable)?.navBarTitle ?? "Back" }
		navBar.updateHistoryMenu(stackTitles: stackTitles, fullCoverTitles: fullTitles)
	}
	
	private func addContent(_ vc: NSViewController) {
		container.addSubview(vc.view)
		activateConstraints(for: vc.view)
	}
	
	private func snapshotView(of view: NSView) -> NSView? {
		guard view.bounds.width > 0, view.bounds.height > 0 else { return nil }
		view.wantsLayer = true
		view.displayIfNeeded()
		guard let rep = view.bitmapImageRepForCachingDisplay(in: view.bounds) else { return nil }
		view.cacheDisplay(in: view.bounds, to: rep)
		let image = NSImage(size: view.bounds.size)
		image.addRepresentation(rep)
		let snapshot = NSImageView(frame: view.frame)
		snapshot.image = image
		snapshot.imageScaling = .scaleAxesIndependently
		snapshot.wantsLayer = true
		return snapshot
	}
	
	private func createDimmingOverlay(frame: NSRect) -> NSView {
		let v = NSView(frame: frame); v.wantsLayer = true
		v.layer?.backgroundColor = NSColor.black.cgColor
		return v
	}
	
	private func removeConstraints(for v: NSView, in host: NSView) {
		host.constraints.filter { ($0.firstItem as? NSView) == v || ($0.secondItem as? NSView) == v }.forEach { host.removeConstraint($0) }
	}
	
	private func removeFullCoverConstraints(for v: NSView) {
		guard let host = view.window?.contentView else { return }
		removeConstraints(for: v, in: host)
	}
	
	private func activateConstraints(for v: NSView) {
		v.translatesAutoresizingMaskIntoConstraints = false
		NSLayoutConstraint.activate([
			v.topAnchor.constraint(equalTo: container.topAnchor),
			v.leadingAnchor.constraint(equalTo: container.leadingAnchor),
			v.trailingAnchor.constraint(equalTo: container.trailingAnchor),
			v.bottomAnchor.constraint(equalTo: container.bottomAnchor)
		])
	}
	
	private func activateFullCoverConstraints(for v: NSView, in host: NSView?) {
		guard let host = host else { return }
		v.translatesAutoresizingMaskIntoConstraints = false
		NSLayoutConstraint.activate([
			v.topAnchor.constraint(equalTo: host.topAnchor),
			v.leadingAnchor.constraint(equalTo: host.leadingAnchor),
			v.trailingAnchor.constraint(equalTo: host.trailingAnchor),
			v.bottomAnchor.constraint(equalTo: host.bottomAnchor)
		])
	}
	
	private func setNavBarInteractionEnabled(_ e: Bool) {
		let current = fullCoverVCs.last ?? stack.last
		if let nav = current as? VNNavigable, let nb = nav.navigationBar {
			nb.backButton.isEnabled = e
			nb.rightStack.arrangedSubviews.forEach { ($0 as? NSControl)?.isEnabled = e }
		}
	}
}
