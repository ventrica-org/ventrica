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
		self.runDeps			= []
	}
	
	init(repoPackage: VentRepoPackage) {
		self.name				= String(cString: repoPackage.name)
		self.version			= String(cString: repoPackage.version)
		self.description		= String(cString: repoPackage.description)
		self.category			= String(cString: repoPackage.category)
		self.addedAt			= nil
		self.icon				= repoPackage.icon.map { String(cString: $0) }
		self.nativeDescription	= repoPackage.native_description.map { String(cString: $0) }
		self.storeName			= String(cString: repoPackage.store_name)
		self.fileName			= String(cString: repoPackage.filename)
		self.varHash			= String(cString: repoPackage.var_hash)
		self.runDeps 			= _cStringArrayToSwift(repoPackage.run_deps, maxCount: Int(repoPackage.run_deps_count))
	}
}

fileprivate func _cStringArrayToSwift(
	_ pointer: UnsafePointer<UnsafePointer<CChar>?>?,
	maxCount: Int
) -> [String] {
	guard let pointer = pointer else { return [] }

	var result: [String] = []

	for i in 0..<maxCount {
		guard let cString = pointer[i] else { break }
		result.append(String(cString: cString))
	}

	return result
}
