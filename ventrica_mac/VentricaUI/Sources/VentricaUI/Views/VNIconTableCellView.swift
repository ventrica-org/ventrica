//
//  VNPackagesViewCell.swift
//  Ventrica
//
//  Created by samsam on 3/8/26.
//

import AppKit
import SwiftUI

public class VNIconTableCellView: NSTableCellView {
	public static let identifier = NSUserInterfaceItemIdentifier("VNIconTableCellView")
	
	private let _iconSize: CGFloat = 32
	
	public let iconView: NSImageView = {
		let v = NSImageView()
		v.imageScaling = .scaleProportionallyUpOrDown
		v.image = NSImage(named: NSImage.applicationIconName)
		v.wantsLayer = true
		v.layer?.cornerRadius = 7.4784
		v.layer?.cornerCurve = .continuous
		v.layer?.masksToBounds = true
		v.layer?.borderWidth = 1
		v.layer?.borderColor = NSColor.gray.withAlphaComponent(0.3).cgColor
		return v
	}()
	
	public let nameLabel: NSTextField = {
		let v = NSTextField(labelWithString: "")
		v.font = .systemFont(ofSize: 13, weight: .semibold)
		v.lineBreakMode = .byTruncatingTail
		return v
	}()
	
	public let descriptionLabel: NSTextField = {
		let v = NSTextField(labelWithString: "")
		v.font = .systemFont(ofSize: 11)
		v.textColor = .secondaryLabelColor
		v.lineBreakMode = .byTruncatingTail
		return v
	}()
	
	public let textStack: NSStackView = {
		let v = NSStackView()
		v.orientation = .vertical
		v.alignment = .leading
		v.spacing = 2
		return v
	}()
	
	override init(frame frameRect: NSRect) {
		super.init(frame: frameRect)
		_setup()
	}
	
	@available(*, unavailable)
	required public init?(coder: NSCoder) {
		fatalError("init(coder:) has not been implemented")
	}
	
	private func _setup() {
		[nameLabel, descriptionLabel].forEach {
			textStack.addArrangedSubview($0)
		}
		
		[iconView, textStack].forEach {
			addSubview($0)
			$0.translatesAutoresizingMaskIntoConstraints = false
		}
		
		NSLayoutConstraint.activate([
			iconView.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 8),
			iconView.topAnchor.constraint(equalTo: topAnchor, constant: 6),
			iconView.widthAnchor.constraint(equalToConstant: _iconSize),
			iconView.heightAnchor.constraint(equalToConstant: _iconSize),
			
			textStack.leadingAnchor.constraint(equalTo: iconView.trailingAnchor, constant: 10),
			textStack.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -8),
			textStack.topAnchor.constraint(equalTo: topAnchor, constant: 6)
		])
	}
	
	internal func configure(title: String, description: String, icon: NSImage?) {
		nameLabel.stringValue = title
		descriptionLabel.stringValue = description
		iconView.image = icon
	}
}

// MARK: - Preview

#Preview(VNIconTableCellView.className()) {
	struct Preview: NSViewRepresentable {
		func makeNSView(context: Context) -> NSView {
			let cell = VNIconTableCellView()
			cell.configure(
				title: "Foo",
				description: "0.0.0 • Foo",
				icon: VNCategoryIdentifier("sources").sectionIcon.image()!
			)
			return cell
		}
		
		func updateNSView(_ nsView: NSView, context: Context) {}
	}

	return HStack {
		List {
			Group {
				ForEach(0..<3) { _ in
					Preview()
				}
			}
			.frame(height: 44)
		}
	}
}
