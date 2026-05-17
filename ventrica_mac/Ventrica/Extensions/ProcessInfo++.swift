//
//  ProcessInfo++.swift
//  Ventrica
//
//  Created by samsam on 3/12/26.
//

import Foundation
import MachO.dyld.utils

extension Foundation.ProcessInfo {
	var architecture: String {
		String(cString: macho_arch_name_for_mach_header(nil)!)
	}
	
	var marketingModel: String {
		MGCopyAnswer(kMGPhysicalHardwareNameString)?.takeUnretainedValue() as? String ?? ""
	}
}
