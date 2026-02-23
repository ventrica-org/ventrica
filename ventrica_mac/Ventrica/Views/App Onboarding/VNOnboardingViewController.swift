import AppKit
import QuartzCore

class VNOnboardingViewController: NSViewController {

	private var package: CAPackage?
	private var animeLayer: CALayer?
	private var stateController: CAStateController?
	private var stateTimer: Timer?
	private var isBig: Bool = false
	

	override func loadView() {
		// Base view
		view = NSView(frame: NSRect(x: 0, y: 0, width: 430, height: 500))
		view.wantsLayer = true

		// Load the CAML bundle
		guard let url = Bundle.main.url(forResource: "onboarding", withExtension: "ca") else {
			print("Failed to find onboarding.ca")
			return
		}

		do {
			let loadedPackage = try CAPackage.package(withContentsOf: url,
													  type: kCAPackageTypeCAMLBundle,
													  options: nil) as? CAPackage
			guard let package = loadedPackage else { return }
			self.package = package

			// Root layer of CAML
			let rootLayer = package.rootLayer

			// Optional: scale down to fit view
			rootLayer!.setAffineTransform(CGAffineTransform(scaleX: 0.4, y: 0.4))

			// Center in view
			rootLayer!.anchorPoint = CGPoint(x: 0.5, y: 0.5)
			rootLayer!.position = CGPoint(x: view.bounds.midX, y: view.bounds.midY)

			// Flip geometry if needed
			if package.isGeometryFlipped {
				rootLayer!.setValue(true, forKey: "geometryFlipped")
			}
			
			self.animeLayer = rootLayer

			self.stateController = CAStateController(layer: self.animeLayer)
			view.layer?.addSublayer(self.animeLayer!)
			self.stateController?.setInitialStatesOfLayer(self.animeLayer, transitionSpeed: 0.0)

		} catch {
			print("Failed to load CAPackage:", error)
		}
	}

	override func viewDidAppear() {
		super.viewDidAppear()

		// Start the state toggling after a short delay to allow initialization
		DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) { [weak self] in
			self?.toggleState() // initial toggle
			self?.startStateTimer()
		}
	}

	private func startStateTimer() {
		guard stateTimer == nil else { return }

		stateTimer = Timer.scheduledTimer(withTimeInterval: 2.0, repeats: true) { [weak self] _ in
			DispatchQueue.main.async {
				self?.toggleState()
			}
		}
	}

	private func toggleState() {
		guard let controller = stateController else { return }
		isBig.toggle()
		
		let a = self.animeLayer!.value(forKey: "states") as! Array<Any>
//		let a[1]
//		print(a)
		
//		print(controller.state(ofLayer: self.animeLayer!))

		if isBig {
			controller.setState(a[0], ofLayer: self.animeLayer!, transitionSpeed: 0.8)
		} else {
			controller.setState(nil, ofLayer: self.animeLayer!, transitionSpeed: 0.8)
		}
		
//		print(controller.state(ofLayer: self.animeLayer!))
//		fatalError()
//		controller._applyTransition(b, layer: controller.layer, undo: nil, speed: 0.5)
		
	}

	override func viewWillDisappear() {
		super.viewWillDisappear()
		stateTimer?.invalidate()
		stateTimer = nil
	}
}
