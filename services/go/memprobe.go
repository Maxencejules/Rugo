package main

import "unsafe"

// Demand-paging probe: touch 16 pages of the kernel's demand window
// (16 MiB..24 MiB) before any service starts. Each first touch must fault,
// be mapped by the kernel, and then behave as ordinary zeroed memory.

const (
	demandProbeBase  = uintptr(0x01000000)
	demandProbePages = 16
	demandPageSize   = uintptr(0x1000)
)

var (
	msgMemProbeOK  = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 'm', 'e', 'm', ' ', 'd', 'e', 'm', 'a', 'n', 'd', ' ', 'o', 'k', '\n'}
	msgMemProbeErr = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 'm', 'e', 'm', ' ', 'd', 'e', 'm', 'a', 'n', 'd', ' ', 'e', 'r', 'r', '\n'}
)

func memDemandProbe() bool {
	var idx uintptr
	for idx = 0; idx < demandProbePages; idx++ {
		p := (*byte)(unsafe.Pointer(demandProbeBase + idx*demandPageSize))
		if *p != 0 {
			return false
		}
		*p = byte(idx + 1)
		if *p != byte(idx+1) {
			return false
		}
	}
	return true
}

func runMemDemandProbe() {
	if memDemandProbe() {
		log(msgMemProbeOK[:])
		return
	}
	log(msgMemProbeErr[:])
}
