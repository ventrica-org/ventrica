//
//  Bundle++.swift
//  VentricaUI
//
//  Created by samsam on 12/30/25.
//

import Foundation

extension Foundation.Bundle {
	/// Get the name of the app
	var name: String {
		if
			let displayName = object(forInfoDictionaryKey: "CFBundleDisplayName") as? String,
			!displayName.isEmpty
		{
			return displayName
		}
		
		if
			let name = object(forInfoDictionaryKey: "CFBundleName") as? String,
			!name.isEmpty
		{
			return name
		}
		
		return exec
	}
	
	/// Get the executable name of the app
	var exec: String {
		object(forInfoDictionaryKey: "CFBundleExecutable") as? String ?? ""
	}
	
	/// Get the "short" version of the app
	var version: String {
		if let version = object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String {
			version
		} else {
			buildVersion
		}
	}
	
	/// Get the "build" version of the app
	var buildVersion: String {
		object(forInfoDictionaryKey: "CFBundleVersion") as? String ?? ""
	}
}
