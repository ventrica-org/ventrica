//
//  VNPackage.swift
//  Ventrica
//
//  Created by samsam on 3/18/26.
//

import VentricaKit

struct Package {
	let name: String
	let version: String
	let description: String
	let category: String
	let icon: String?
	let nativeDescription: String?
	let addedAt: Int64?
	let storeName: String?
	let fileName: String?
	let varHash: String?
	let runDeps: [String]
	
	init(package: VentPackage) {
		self.name				= String(cString: package.name)
		self.version			= String(cString: package.version)
		self.description		= String(cString: package.description)
		self.category			= String(cString: package.category)
		self.addedAt			= package.installed_at
		self.icon				= package.icon.map { String(cString: $0) }
		self.nativeDescription	= package.native_description.map { String(cString: $0) }
		self.storeName			= nil
		self.fileName			= nil
		self.varHash			= nil
		self.runDeps			= cStringArrayToSwift(package.run_dep_names, maxCount: Int(package.run_dep_names_count))
	}
}

func cStringArrayToSwift(
	_ pointer: UnsafePointer<UnsafePointer<CChar>?>?,
	maxCount: Int
) -> [String] {
	guard
		let pointer,
		maxCount > 0
	else {
		return []
	}

	var result: [String] = []

	for i in 0..<maxCount {
		guard let cString = pointer[i] else { break }
		result.append(String(cString: cString))
	}

	return result
}
