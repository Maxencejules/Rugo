package main

var (
	msgShellPrompt        = []byte("rugo> ")
	msgShellSessionReady  = []byte("GOSH: session ready\n")
	msgShellHelp          = []byte("GOSH: commands help status time storage netcheck health pkg apps run <app> crash shutdown\n")
	msgShellUnknown       = []byte("GOSH: unknown\n")
	msgShellPromptErr     = []byte("GOSH: prompt err\n")
	msgShellReadErr       = []byte("GOSH: read err=")
	msgShellEchoErr       = []byte("GOSH: echo err\n")
	msgShellNewlineErr    = []byte("GOSH: newline err\n")
	msgShellBackspaceErr  = []byte("GOSH: backspace err\n")
	msgShellShutdown      = []byte("GOSH: shutdown\n")
	msgShellCrash         = []byte("GOSH: crash\n")
	msgShellPkgMissing    = []byte("APP: package state unavailable\n")
	msgShellAppBase       = []byte("APP: base-shell installed\n")
	msgShellAppNet        = []byte("APP: net-tools installed\n")
	msgShellAppMedia      = []byte("APP: media-suite installed\n")
	msgShellAppBaseMiss   = []byte("APP: base-shell missing\n")
	msgShellAppNetMiss    = []byte("APP: net-tools missing\n")
	msgShellAppMediaMiss  = []byte("APP: media-suite missing\n")
	msgShellAppExecErr    = []byte("APP: base-shell exec err\n")
	msgShellAppBaseOK     = []byte("APP: base-shell ok\n")
	appNameBaseShell      = []byte("base-shell")
	msgShellAppNetOK      = []byte("APP: net-tools ok\n")
	msgShellAppMediaOK    = []byte("APP: media-suite ok\n")
	msgShellBackspaceEcho = []byte{8, ' ', 8}
	msgShellNewline       = []byte{'\r', '\n'}
)

func shellSession(replyEP uintptr, timeEP uintptr, diagEP uintptr, pkgEP uintptr) bool {
	log(msgShellSessionReady)

	var line [96]byte
	for {
		if !consoleWriteBytes(msgShellPrompt) {
			log(msgShellPromptErr)
			return false
		}

		n, ok := consoleReadLine(line[:])
		if !ok {
			return false
		}
		if n == 0 {
			continue
		}

		shouldExit, commandOK := shellHandleCommand(string(line[:n]), replyEP, timeEP, diagEP, pkgEP)
		if !commandOK {
			log(msgShellErr[:])
		}
		if shouldExit {
			return commandOK
		}
	}
}

func consoleWriteBytes(msg []byte) bool {
	if len(msg) == 0 {
		return true
	}
	return sysWrite(1, &msg[0], uintptr(len(msg))) == uintptr(len(msg))
}

func consoleReadLine(buf []byte) (uintptr, bool) {
	var idx uintptr
	for {
		var ch [1]byte
		n := sysRead(0, &ch[0], 1)
		if n != 1 {
			var lb lineBuilder
			lb.add(msgShellReadErr)
			lb.addUint(n)
			lb.add(msgShellNewline)
			lb.emit()
			return 0, false
		}

		switch ch[0] {
		case '\r', '\n':
			if !consoleWriteBytes(msgShellNewline) {
				log(msgShellNewlineErr)
				return 0, false
			}
			return idx, true
		case 8, 127:
			if idx == 0 {
				continue
			}
			idx--
			if !consoleWriteBytes(msgShellBackspaceEcho) {
				log(msgShellBackspaceErr)
				return 0, false
			}
		default:
			if idx >= uintptr(len(buf)) {
				continue
			}
			buf[idx] = ch[0]
			idx++
			if sysWrite(1, &ch[0], 1) != 1 {
				log(msgShellEchoErr)
				return 0, false
			}
		}
	}
}

func shellHandleCommand(cmd string, replyEP uintptr, timeEP uintptr, diagEP uintptr, pkgEP uintptr) (bool, bool) {
	// File-tree builtins take arguments; dispatch on prefix.
	if len(cmd) > 8 && cmd[:8] == "fswrite " {
		return false, fshWrite(cmd[8:])
	}
	if len(cmd) > 6 && cmd[:6] == "fscat " {
		return false, fshCat(cmd[6:])
	}
	if len(cmd) > 5 && cmd[:5] == "fsls " {
		return false, fshLs(cmd[5:])
	}
	if len(cmd) > 5 && cmd[:5] == "fsmk " {
		return false, fshCtl(fsCtlMkdir, cmd[5:], msgFshMkOK)
	}
	if len(cmd) > 5 && cmd[:5] == "fsrm " {
		return false, fshCtl(fsCtlUnlink, cmd[5:], msgFshRmOK)
	}
	switch cmd {
	case "help":
		log(msgShellHelp)
		return false, true
	case "status":
		if !requestDiag(diagEP, replyEP) {
			return false, false
		}
		log(msgShellDiag[:])
		return false, true
	case "time":
		if !requestTime(timeEP, replyEP) {
			return false, false
		}
		log(msgShellReply[:])
		return false, true
	case "storage":
		return false, runC4Storage()
	case "netcheck":
		return false, runC4Network()
	case "health":
		return false, runHealthCheck(replyEP, timeEP, diagEP, pkgEP)
	case "pkg":
		if pkgEP == sysErr {
			log(msgShellPkgMissing)
			return false, true
		}
		if !requestPkg(pkgEP, replyEP) {
			return false, false
		}
		log(msgShellPkg[:])
		return false, true
	case "apps":
		return false, listInstalledApps()
	case "run base-shell":
		return false, runInstalledApp("base-shell")
	case "run net-tools":
		return false, runInstalledApp("net-tools")
	case "run media-suite":
		return false, runInstalledApp("media-suite")
	case "crash":
		log(msgShellCrash)
		markServiceFailed(serviceShell)
		sysThreadExit()
		haltForever()
		return false, false
	case "exit", "shutdown":
		log(msgShellShutdown)
		return true, true
	default:
		log(msgShellUnknown)
		return false, true
	}
}

func runHealthCheck(replyEP uintptr, timeEP uintptr, diagEP uintptr, pkgEP uintptr) bool {
	if !runShellPolicyChecks(timeEP) {
		return false
	}
	if !requestTime(timeEP, replyEP) {
		return false
	}
	log(msgShellReply[:])
	if !requestDiag(diagEP, replyEP) {
		return false
	}
	log(msgShellDiag[:])
	if !runC4Storage() || !runC4Network() {
		return false
	}
	if !runC5Isolation(replyEP, diagEP) || !runC5Reliability(replyEP, timeEP) {
		return false
	}
	if desktopProfileEnabled {
		if !runDesktopProfile(replyEP, diagEP) {
			return false
		}
	}
	if pkgEP != sysErr {
		if !requestPkg(pkgEP, replyEP) {
			return false
		}
		log(msgShellPkg[:])
	}
	return true
}

func runShellPolicyChecks(timeEP uintptr) bool {
	if timeEP == sysErr {
		return false
	}
	log(msgShellLookup[:])

	var deny [1]byte
	if sysIpcRecv(timeEP, &deny[0], uintptr(len(deny))) != sysErr {
		return false
	}
	log(msgShellRecvDeny[:])

	if sysSvcRegister(&nameHijack[0], uintptr(len(nameHijack)), timeEP) != sysErr {
		return false
	}
	log(msgShellRegDeny[:])

	if sysThreadSpawn(spawnEntry) != sysErr {
		return false
	}
	log(msgShellSpawnDeny[:])
	return true
}

func listInstalledApps() bool {
	var state pkgState
	if !loadPkgState(&state) {
		log(msgShellPkgMissing)
		return true
	}

	if state.InstalledMask&pkgInstallBaseShell != 0 {
		log(msgShellAppBase)
	} else {
		log(msgShellAppBaseMiss)
	}
	if state.InstalledMask&pkgInstallNetTools != 0 {
		log(msgShellAppNet)
	} else {
		log(msgShellAppNetMiss)
	}
	if state.InstalledMask&pkgInstallMediaSuite != 0 {
		log(msgShellAppMedia)
	} else {
		log(msgShellAppMediaMiss)
	}
	return true
}

func runInstalledApp(name string) bool {
	var state pkgState
	if !loadPkgState(&state) {
		log(msgShellPkgMissing)
		return true
	}

	switch name {
	case "base-shell":
		if state.InstalledMask&pkgInstallBaseShell == 0 {
			log(msgShellAppBaseMiss)
			return true
		}
		// Real execution: load the app ELF from the package store on
		// disk, run it as a child task, and reap it.
		tid := sysSpawn(&appNameBaseShell[0], uintptr(len(appNameBaseShell)))
		if tid == sysErr {
			log(msgShellAppExecErr)
			return false
		}
		if sysWait(tid, nil, 0) == sysErr {
			log(msgShellAppExecErr)
			return false
		}
		log(msgShellAppBaseOK)
		return true
	case "net-tools":
		if state.InstalledMask&pkgInstallNetTools == 0 {
			log(msgShellAppNetMiss)
			return true
		}
		if !runSocketRoundTrip(4041, c4TcpPing[:], false) {
			return false
		}
		log(msgShellAppNetOK)
		return true
	case "media-suite":
		if state.InstalledMask&pkgInstallMediaSuite == 0 {
			log(msgShellAppMediaMiss)
			return true
		}
		log(msgShellAppMediaOK)
		return true
	default:
		log(msgShellUnknown)
		return true
	}
}
