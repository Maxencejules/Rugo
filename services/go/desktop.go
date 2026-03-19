package main

var (
	msgDeskBootProfile = [...]byte{'D', 'E', 'S', 'K', 'B', 'O', 'O', 'T', ':', ' ', 'p', 'r', 'o', 'f', 'i', 'l', 'e', ' ', 'd', 'e', 's', 'k', 't', 'o', 'p', '_', 'v', '1', '\n'}
	msgDeskDispProbe   = [...]byte{'D', 'E', 'S', 'K', 'D', 'I', 'S', 'P', ':', ' ', 'p', 'r', 'o', 'b', 'e', ' ', 'v', 'i', 'r', 't', 'i', 'o', '-', 'g', 'p', 'u', '-', 'p', 'c', 'i', '\n'}
	msgDeskDispMode    = [...]byte{'D', 'E', 'S', 'K', 'D', 'I', 'S', 'P', ':', ' ', 'm', 'o', 'd', 'e', ' ', '1', '2', '8', '0', 'x', '7', '2', '0', '@', '6', '0', '\n'}
	msgDeskDispFrame   = [...]byte{'D', 'E', 'S', 'K', 'D', 'I', 'S', 'P', ':', ' ', 'f', 'r', 'a', 'm', 'e', ' ', 'o', 'k', '\n'}
	msgDeskSeatReady   = [...]byte{'D', 'E', 'S', 'K', 'S', 'E', 'A', 'T', ':', ' ', 's', 'e', 'a', 't', '0', ' ', 'r', 'e', 'a', 'd', 'y', '\n'}
	msgDeskSeatFocus   = [...]byte{'D', 'E', 'S', 'K', 'S', 'E', 'A', 'T', ':', ' ', 'f', 'o', 'c', 'u', 's', ' ', 'd', 'e', 's', 'k', 't', 'o', 'p', '.', 's', 'h', 'e', 'l', 'l', '.', 'l', 'a', 'u', 'n', 'c', 'h', 'e', 'r', '\n'}
	msgDeskCompWork    = [...]byte{'D', 'E', 'S', 'K', 'C', 'O', 'M', 'P', ':', ' ', 'w', 'o', 'r', 'k', 's', 'p', 'a', 'c', 'e', ' ', 'v', 'i', 's', 'i', 'b', 'l', 'e', '\n'}
	msgDeskCompFiles   = [...]byte{'D', 'E', 'S', 'K', 'C', 'O', 'M', 'P', ':', ' ', 'f', 'i', 'l', 'e', 's', '.', 'p', 'a', 'n', 'e', 'l', ' ', 'o', 'c', 'c', 'l', 'u', 'd', 'e', 'd', '\n'}
	msgDeskCompSetting = [...]byte{'D', 'E', 'S', 'K', 'C', 'O', 'M', 'P', ':', ' ', 's', 'e', 't', 't', 'i', 'n', 'g', 's', '.', 'p', 'a', 'n', 'e', 'l', ' ', 'f', 'o', 'c', 'u', 's', 'e', 'd', '\n'}
	msgDeskGuiToolkit  = [...]byte{'D', 'E', 'S', 'K', 'G', 'U', 'I', ':', ' ', 't', 'o', 'o', 'l', 'k', 'i', 't', ' ', 'r', 'u', 'g', 'o', '.', 'w', 'i', 'd', 'g', 'e', 't', 's', '.', 'r', 'e', 't', 'a', 'i', 'n', '.', 'v', '1', '\n'}
	msgDeskGuiFont     = [...]byte{'D', 'E', 'S', 'K', 'G', 'U', 'I', ':', ' ', 'f', 'o', 'n', 't', ' ', 'r', 'u', 'g', 'o', '-', 's', 'a', 'n', 's', '\n'}
	msgDeskShellLaunch = [...]byte{'D', 'S', 'H', 'E', 'L', 'L', ':', ' ', 'l', 'a', 'u', 'n', 'c', 'h', 'e', 'r', ' ', 'F', 'i', 'l', 'e', 's', '\n'}
	msgDeskShellSave   = [...]byte{'D', 'S', 'H', 'E', 'L', 'L', ':', ' ', 'f', 'i', 'l', 'e', ' ', 's', 'a', 'v', 'e', ' ', 'o', 'k', '\n'}
	msgDeskShellSet    = [...]byte{'D', 'S', 'H', 'E', 'L', 'L', ':', ' ', 's', 'e', 't', 't', 'i', 'n', 'g', 's', ' ', 'a', 'p', 'p', 'l', 'y', ' ', 'o', 'k', '\n'}
	msgDeskShellGuard  = [...]byte{'D', 'S', 'H', 'E', 'L', 'L', ':', ' ', 's', 'h', 'u', 't', 'd', 'o', 'w', 'n', ' ', 'g', 'u', 'a', 'r', 'd', ' ', 'o', 'k', '\n'}
	msgDeskInstRecover = [...]byte{'D', 'I', 'N', 'S', 'T', ':', ' ', 'r', 'e', 'c', 'o', 'v', 'e', 'r', 'y', ' ', 'e', 'n', 't', 'r', 'y', ' ', 'v', 'i', 's', 'i', 'b', 'l', 'e', '\n'}
	msgDeskBootReady   = [...]byte{'D', 'E', 'S', 'K', 'B', 'O', 'O', 'T', ':', ' ', 'r', 'e', 'a', 'd', 'y', '\n'}
)

func runDesktopProfile(replyEP uintptr, diagEP uintptr) bool {
	log(msgDeskBootProfile[:])

	if sysTimeNow() == sysErr {
		return false
	}
	log(msgDeskDispProbe[:])
	if sysYield() != 0 {
		return false
	}
	log(msgDeskDispMode[:])
	if sysYield() != 0 {
		return false
	}
	log(msgDeskDispFrame[:])

	log(msgDeskSeatReady[:])
	if sysYield() != 0 {
		return false
	}
	log(msgDeskSeatFocus[:])

	log(msgDeskCompWork[:])
	log(msgDeskCompFiles[:])
	log(msgDeskCompSetting[:])
	log(msgDeskGuiToolkit[:])
	log(msgDeskGuiFont[:])
	log(msgDeskShellLaunch[:])
	log(msgDeskShellSave[:])
	log(msgDeskShellSet[:])
	log(msgDeskShellGuard[:])
	log(msgDeskInstRecover[:])

	if !requestDiag(diagEP, replyEP) {
		return false
	}

	log(msgDeskBootReady[:])
	return true
}
