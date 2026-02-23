//
//  VentricaApp.swift
//  Ventrica
//
//  Created by samsam on 12/23/25.
//

import AppKit
import VentricaKit

@main enum VNEntry {
	static func main() {
		let delegate = VNAppDelegate()
		NSApplication.shared.delegate = delegate
		_ = NSApplicationMain(CommandLine.argc, CommandLine.unsafeArgv)
	}
}
