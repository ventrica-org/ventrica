//
//  Bundle++.swift
//  VentricaUI
//
//  Created by samsam on 12/30/25.
//

import Foundation

extension Foundation.Bundle {
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
	
	var exec: String {
		object(forInfoDictionaryKey: "CFBundleExecutable") as? String ?? ""
	}
	
	var version: String {
		if let version = object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String {
			version
		} else {
			buildVersion
		}
	}
	
	var buildVersion: String {
		object(forInfoDictionaryKey: "CFBundleVersion") as? String ?? ""
	}
}
