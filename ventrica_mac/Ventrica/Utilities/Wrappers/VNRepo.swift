//
//  VNRepo.swift
//  Ventrica
//
//  Created by samsam on 3/18/26.
//

import VentricaKit

struct VNRepo {
	let name: String
	let url: String
	let addedAt: Int64
	
	init(repo: VentRepo) {
		self.name 		= String(cString: repo.name)
		self.url 		= String(cString: repo.url)
		self.addedAt 	= repo.added_at
	}
}
