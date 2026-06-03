//
//  VNRepo.swift
//  Ventrica
//
//  Created by samsam on 3/18/26.
//

import VentricaKit

struct Repo {
	let name: String
	let url: String?
	let icon: String?
	let addedAt: Int64
	
	init(repo: VentRepo) {
		self.name 		= String(cString: repo.name)
		self.url 		= repo.url.map { String(cString: $0) }
		self.icon 		= repo.icon.map { String(cString: $0) }
		self.addedAt 	= repo.added_at
	}
}
