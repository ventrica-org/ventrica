//
//  VentricaApp.swift
//  Ventrica
//
//  Created by samsam on 12/23/25.
//

import AppKit
import VentricaKit

@main enum Entry {
	static func main() {
		_ = InstallQueue.shared
		
		let delegate = AppDelegate()
		NSApplication.shared.delegate = delegate
		_ = NSApplicationMain(CommandLine.argc, CommandLine.unsafeArgv)
	}
}
