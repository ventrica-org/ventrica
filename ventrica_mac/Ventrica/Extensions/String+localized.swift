//
//  String+localized.swift
//  NimbleKit
//
//  Created by samara on 20.03.2025.
//

extension String {
	static public func localized(_ name: String) -> String {
		NSLocalizedString(name, comment: "")
	}
	
	static public func localized(_ name: String, arguments: CVarArg...) -> String {
		String(format: NSLocalizedString(name, comment: ""), arguments: arguments)
	}
	/// Localizes the current string using the main bundle.
	///
	/// - Returns: The localized string.
	public func localized() -> String {
		String.localized(self)
	}
}
