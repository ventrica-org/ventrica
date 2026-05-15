//
//  ImageLoader.swift
//  Ventrica
//
//  Created by samsam on 3/11/26.
//

import AppKit

final class ImageLoader {
	@MainActor static let shared = ImageLoader()
	
	private let _memoryCache = NSCache<NSString, NSImage>()
	private let _cacheDirectory: URL
	
	private init() {
		_memoryCache.countLimit = 100
		
		let base = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask)[0]
		_cacheDirectory = base.appendingPathComponent(Bundle.main.bundleIdentifier!, isDirectory: true)
		
		try? FileManager.default.createDirectory(at: _cacheDirectory, withIntermediateDirectories: true)
	}
	
	func load(url: URL) async -> NSImage? {
		let key = url.absoluteString as NSString
		
		if let img = _memoryCache.object(forKey: key) {
			return img
		}
		
		let file = _cacheDirectory.appendingPathComponent(url.lastPathComponent)
		
		if
			let data = try? Data(contentsOf: file),
			let img = NSImage(data: data)
		{
			_memoryCache.setObject(img, forKey: key)
			return img
		}
		
		guard
			let (data, _) = try? await URLSession.shared.data(from: url),
			let img = NSImage(data: data)
		else {
			return nil
		}
		
		_memoryCache.setObject(img, forKey: key)
		try? data.write(to: file)
		
		return img
	}
}
