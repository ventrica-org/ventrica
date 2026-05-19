//
//  PackagesCollectionViewController.swift
//  Ventrica
//
//  Created by samsam on 5/18/26.
//

import AppKit
import VentricaUI
import VentricaKit

protocol PackagesCollectionViewControllerDelegate: AnyObject {
	func packagesCollectionViewController(_ vc: PackagesCollectionViewController, didSelect package: Package?)
}

final class PackagesCollectionViewController: NSViewController {
	weak var delegate: PackagesCollectionViewControllerDelegate?
	
	private let _scrollView = NSScrollView()
	private let _collectionView = NSCollectionView()
	private var _packages: [Package] = []
	private var _url: String?
	
	private static let _itemIdentifier = NSUserInterfaceItemIdentifier("PackageGridItem")
	
	private var _isLastRow: Bool = false
	
	init(titleText: String, url: String?) {
		self._url = url
		super.init(nibName: nil, bundle: nil)
		self.title = titleText
	}
	
	@available(*, unavailable)
	required init?(coder: NSCoder) { fatalError() }
	
	override func loadView() {
		view = .init()
		_setupCollectionView()
		_setupListeners()
	}
	
	private func _setupCollectionView() {
		let layout = NSCollectionViewFlowLayout()
		layout.scrollDirection = .vertical
		layout.minimumInteritemSpacing = 0
		layout.minimumLineSpacing = 0
		layout.sectionInset = NSEdgeInsets(top: 20, left: 20, bottom: 20, right: 20)
		
		layout.itemSize = NSSize(width: 200, height: 72)
		
		_collectionView.collectionViewLayout = layout
		_collectionView.dataSource = self
		_collectionView.delegate = self
		_collectionView.isSelectable = true
		_collectionView.register(PackageGridItem.self, forItemWithIdentifier: Self._itemIdentifier)
		_collectionView.wantsLayer = true
		
		_scrollView.documentView = _collectionView
		_scrollView.hasVerticalScroller = true
		_scrollView.drawsBackground = false
		_scrollView.translatesAutoresizingMaskIntoConstraints = false
		
		view.addSubview(_scrollView)
		
		NSLayoutConstraint.activate([
			_scrollView.topAnchor.constraint(equalTo: view.topAnchor),
			_scrollView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
			_scrollView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
			_scrollView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
		])
	}
	override func viewDidLayout() {
		super.viewDidLayout()
		_updateItemSizeForWidth()
	}
	
	private func _updateItemSizeForWidth() {
		guard let layout = _collectionView.collectionViewLayout as? NSCollectionViewFlowLayout else { return }
		let sectionInset = layout.sectionInset
		let spacing = layout.minimumInteritemSpacing
		let maxColumns = 5
		let availableWidth = view.bounds.width - sectionInset.left - sectionInset.right

		var columns = maxColumns
		while columns >= 1 {
			let totalSpacing = CGFloat(columns - 1) * spacing
			let candidateWidth = (availableWidth - totalSpacing) / CGFloat(columns)
			if candidateWidth >= 270 {
				layout.itemSize = NSSize(width: candidateWidth, height: 72)
				_collectionView.collectionViewLayout?.invalidateLayout()
				return
			}
			columns -= 1
		}
		
		layout.itemSize = NSSize(width: max(availableWidth, 100), height: 72)
		_collectionView.collectionViewLayout?.invalidateLayout()
	}
	
	private func _setupListeners() {
		_load()
		
		NotificationCenter.default.addObservers(
			[NSApplication.didBecomeActiveNotification, .shouldRefreshPackageList],
			observer: self,
			selector: #selector(_load)
		)
	}
	
	@objc private func _load() {
		var packages: [Package] = []
		var err: OpaquePointer? = nil
		
		guard let store = ventrica_store_open_default(&err) else {
			if let e = err {
				print(String(cString: ventrica_error_message(e)))
				ventrica_error_free(e)
			}
			return
		}
		
		defer { ventrica_store_close(store) }
		
		if let url = _url {
			var pkgArr: UnsafeMutablePointer<UnsafeMutablePointer<VentRepoPackage>?>? = nil
			var pkgCount: Int = 0
			
			guard ventrica_list_repo_packages(store, url, &pkgArr, &pkgCount, &err) == 0 else {
				if let e = err {
					print(String(cString: ventrica_error_message(e)))
					ventrica_error_free(e)
				}
				return
			}
			
			if let pkgArr {
				defer { ventrica_repo_package_array_free(pkgArr, UInt(pkgCount)) }
				for i in 0..<pkgCount {
					guard let pkg = pkgArr[i] else { continue }
					packages.append(Package(repoPackage: pkg.pointee))
				}
			}
		} else {
			var arr: UnsafeMutablePointer<UnsafeMutablePointer<VentPackage>?>? = nil
			var count: Int = 0
			
			guard ventrica_list_packages(store, &arr, &count, &err) == 0 else {
				if let e = err {
					print(String(cString: ventrica_error_message(e)))
					ventrica_error_free(e)
				}
				return
			}
			
			if let arr {
				defer { ventrica_pkg_array_free(arr, UInt(count)) }
				for i in 0..<count {
					guard let pkg = arr[i] else { continue }
					packages.append(Package(package: pkg.pointee))
				}
			}
		}
		
		_packages = packages.sorted {
			$0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending
		}
		
		_collectionView.reloadData()
	}
}

extension PackagesCollectionViewController: NSCollectionViewDataSource, NSCollectionViewDelegate {
	func collectionView(_ collectionView: NSCollectionView, numberOfItemsInSection section: Int) -> Int {
		_packages.count
	}
	
	func collectionView(_ collectionView: NSCollectionView, itemForRepresentedObjectAt indexPath: IndexPath) -> NSCollectionViewItem {
		let item = collectionView.makeItem(withIdentifier: Self._itemIdentifier, for: indexPath) as! PackageGridItem
		guard let layout = collectionView.collectionViewLayout as? NSCollectionViewFlowLayout else {
			item.configure(package: _packages[indexPath.item], isLastRow: false)
			return item
		}
		let sectionInset = layout.sectionInset
		let spacing = layout.minimumInteritemSpacing
		let availableWidth = collectionView.bounds.width - sectionInset.left - sectionInset.right
		let cellWidth = layout.itemSize.width
		let columns = max(1, Int((availableWidth + spacing) / (cellWidth + spacing)))
		let row = indexPath.item / columns
		let lastRow = (_packages.count - 1) / columns
		let isLastRow = row == lastRow
		item.configure(package: _packages[indexPath.item], isLastRow: isLastRow)
		return item
	}
	
	func collectionView(_ collectionView: NSCollectionView, didSelectItemsAt indexPaths: Set<IndexPath>) {
		guard let index = indexPaths.first?.item else { return }
		delegate?.packagesCollectionViewController(self, didSelect: _packages[index])
	}
	
	func collectionView(_ collectionView: NSCollectionView, didDeselectItemsAt indexPaths: Set<IndexPath>) {
		if collectionView.selectionIndexPaths.isEmpty {
			delegate?.packagesCollectionViewController(self, didSelect: nil)
		}
	}
}

private final class PackageGridItem: NSCollectionViewItem {
	private let _iconSize: CGFloat = 48
	
	private let _iconView: NSImageView = {
		let v = NSImageView()
		v.imageScaling = .scaleProportionallyUpOrDown
		v.wantsLayer = true
		v.layer?.cornerRadius = 11
		v.layer?.cornerCurve = .continuous
		v.layer?.masksToBounds = true
		v.layer?.borderWidth = 1
		v.layer?.borderColor = NSColor.gray.withAlphaComponent(0.3).cgColor
		v.translatesAutoresizingMaskIntoConstraints = false
		return v
	}()
	
	private let _nameLabel: NSTextField = {
		let v = NSTextField(labelWithString: "")
		v.font = .systemFont(ofSize: 13, weight: .semibold)
		v.lineBreakMode = .byTruncatingTail
		v.maximumNumberOfLines = 1
		v.translatesAutoresizingMaskIntoConstraints = false
		return v
	}()
	
	private let _descriptionLabel: NSTextField = {
		let v = NSTextField(labelWithString: "")
		v.font = .systemFont(ofSize: 11)
		v.textColor = .secondaryLabelColor
		v.lineBreakMode = .byTruncatingTail
		v.maximumNumberOfLines = 1
		v.translatesAutoresizingMaskIntoConstraints = false
		return v
	}()
	
	private let _separator = VNSeperatorView()
	
	override func loadView() {
		view = NSView()
		view.wantsLayer = true
		
		let textStack = NSStackView(views: [_nameLabel, _descriptionLabel])
		textStack.orientation = .vertical
		textStack.alignment = .leading
		textStack.spacing = 2
		textStack.translatesAutoresizingMaskIntoConstraints = false
		
		view.addSubview(_iconView)
		view.addSubview(textStack)
		_separator.translatesAutoresizingMaskIntoConstraints = false
		view.addSubview(_separator)
		
		NSLayoutConstraint.activate([
			_iconView.leadingAnchor.constraint(equalTo: view.leadingAnchor, constant: 12),
			_iconView.centerYAnchor.constraint(equalTo: view.centerYAnchor),
			_iconView.widthAnchor.constraint(equalToConstant: _iconSize),
			_iconView.heightAnchor.constraint(equalToConstant: _iconSize),
			
			textStack.leadingAnchor.constraint(equalTo: _iconView.trailingAnchor, constant: 12),
			textStack.trailingAnchor.constraint(equalTo: view.trailingAnchor, constant: -12),
			textStack.centerYAnchor.constraint(equalTo: view.centerYAnchor),
			
			_separator.leadingAnchor.constraint(equalTo: _iconView.trailingAnchor, constant: 12),
			_separator.trailingAnchor.constraint(equalTo: view.trailingAnchor, constant: -12),
			_separator.bottomAnchor.constraint(equalTo: view.bottomAnchor),
		])
	}
	
	func configure(package: Package, isLastRow: Bool = false) {
		_nameLabel.stringValue = package.name
		_descriptionLabel.stringValue = "\(package.version) • \(package.description)"
		_iconView.image = VNCategoryIdentifier(package.category).sectionIcon.image()
		
		if let iconString = package.icon, let url = URL(string: iconString) {
			Task { [weak self] in
				guard let self, let image = await ImageLoader.shared.load(url: url) else { return }
				await MainActor.run { self._iconView.image = image }
			}
		}
		_separator.isHidden = isLastRow
	}
}
