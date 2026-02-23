//
//  ProcessInfo++.swift
//  Ventrica
//
//  Created by samsam on 3/12/26.
//

import MachO.dyld.utils

extension Foundation.ProcessInfo {
	/// The devices architecture (e.g. arm64).
	var architecture: String {
		String(cString: macho_arch_name_for_mach_header(nil)!)
	}
	
	/// The devices marketing model (e.g. iPhone 7)
	var marketingModel: String {
		MGCopyAnswer(kMGPhysicalHardwareNameString)?.takeUnretainedValue() as? String ?? ""
	}
}
