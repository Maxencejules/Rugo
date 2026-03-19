package main

var (
	msgTimeSvcStart = [...]byte{'T', 'I', 'M', 'E', 'S', 'V', 'C', ':', ' ', 's', 't', 'a', 'r', 't', '\n'}
	msgTimeSvcReady = [...]byte{'T', 'I', 'M', 'E', 'S', 'V', 'C', ':', ' ', 'r', 'e', 'a', 'd', 'y', '\n'}
	msgTimeSvcReq   = [...]byte{'T', 'I', 'M', 'E', 'S', 'V', 'C', ':', ' ', 'r', 'e', 'q', ' ', 'o', 'k', '\n'}
	msgTimeSvcTime  = [...]byte{'T', 'I', 'M', 'E', 'S', 'V', 'C', ':', ' ', 't', 'i', 'm', 'e', ' ', 'o', 'k', '\n'}
	msgTimeSvcErr   = [...]byte{'T', 'I', 'M', 'E', 'S', 'V', 'C', ':', ' ', 'e', 'r', 'r', '\n'}

	msgDiagSvcStart = [...]byte{'D', 'I', 'A', 'G', 'S', 'V', 'C', ':', ' ', 's', 't', 'a', 'r', 't', '\n'}
	msgDiagSvcReady = [...]byte{'D', 'I', 'A', 'G', 'S', 'V', 'C', ':', ' ', 'r', 'e', 'a', 'd', 'y', '\n'}
	msgDiagSvcSnap  = [...]byte{'D', 'I', 'A', 'G', 'S', 'V', 'C', ':', ' ', 's', 'n', 'a', 'p', 's', 'h', 'o', 't', '\n'}
	msgDiagSvcStop  = [...]byte{'D', 'I', 'A', 'G', 'S', 'V', 'C', ':', ' ', 's', 't', 'o', 'p', '\n'}
	msgDiagSvcErr   = [...]byte{'D', 'I', 'A', 'G', 'S', 'V', 'C', ':', ' ', 'e', 'r', 'r', '\n'}

	msgShellStart     = [...]byte{'G', 'O', 'S', 'H', ':', ' ', 's', 't', 'a', 'r', 't', '\n'}
	msgShellRecycle   = [...]byte{'G', 'O', 'S', 'H', ':', ' ', 'r', 'e', 'c', 'y', 'c', 'l', 'e', '\n'}
	msgShellLookup    = [...]byte{'G', 'O', 'S', 'H', ':', ' ', 'l', 'o', 'o', 'k', 'u', 'p', ' ', 'o', 'k', '\n'}
	msgShellRecvDeny  = [...]byte{'G', 'O', 'S', 'H', ':', ' ', 'r', 'e', 'c', 'v', ' ', 'd', 'e', 'n', 'y', '\n'}
	msgShellRegDeny   = [...]byte{'G', 'O', 'S', 'H', ':', ' ', 'r', 'e', 'g', ' ', 'd', 'e', 'n', 'y', '\n'}
	msgShellSpawnDeny = [...]byte{'G', 'O', 'S', 'H', ':', ' ', 's', 'p', 'a', 'w', 'n', ' ', 'd', 'e', 'n', 'y', '\n'}
	msgShellReply     = [...]byte{'G', 'O', 'S', 'H', ':', ' ', 'r', 'e', 'p', 'l', 'y', ' ', 'o', 'k', '\n'}
	msgShellDiag      = [...]byte{'G', 'O', 'S', 'H', ':', ' ', 'd', 'i', 'a', 'g', ' ', 'o', 'k', '\n'}
	msgShellErr       = [...]byte{'G', 'O', 'S', 'H', ':', ' ', 'e', 'r', 'r', '\n'}

	msgStorC4Stage = [...]byte{'S', 'T', 'O', 'R', 'C', '4', ':', ' ', 'j', 'o', 'u', 'r', 'n', 'a', 'l', ' ', 's', 't', 'a', 'g', 'e', 'd', '\n'}
	msgStorC4State = [...]byte{'S', 'T', 'O', 'R', 'C', '4', ':', ' ', 's', 't', 'a', 't', 'e', ' ', 'o', 'k', '\n'}
	msgStorC4Fsync = [...]byte{'S', 'T', 'O', 'R', 'C', '4', ':', ' ', 'f', 's', 'y', 'n', 'c', ' ', 'o', 'k', '\n'}

	msgNetC4IfCfg   = [...]byte{'N', 'E', 'T', 'C', '4', ':', ' ', 'i', 'f', 'c', 'f', 'g', ' ', 'o', 'k', '\n'}
	msgNetC4Route   = [...]byte{'N', 'E', 'T', 'C', '4', ':', ' ', 'r', 'o', 'u', 't', 'e', ' ', 'o', 'k', '\n'}
	msgNetC4Listen  = [...]byte{'N', 'E', 'T', 'C', '4', ':', ' ', 'l', 'i', 's', 't', 'e', 'n', ' ', 'o', 'k', '\n'}
	msgNetC4Connect = [...]byte{'N', 'E', 'T', 'C', '4', ':', ' ', 'c', 'o', 'n', 'n', 'e', 'c', 't', ' ', 'o', 'k', '\n'}
	msgNetC4Accept  = [...]byte{'N', 'E', 'T', 'C', '4', ':', ' ', 'a', 'c', 'c', 'e', 'p', 't', ' ', 'o', 'k', '\n'}
	msgNetC4Recv    = [...]byte{'N', 'E', 'T', 'C', '4', ':', ' ', 'r', 'e', 'c', 'v', ' ', 'o', 'k', '\n'}
	msgNetC4Reply   = [...]byte{'N', 'E', 'T', 'C', '4', ':', ' ', 'r', 'e', 'p', 'l', 'y', ' ', 'o', 'k', '\n'}

	msgIsoC5Domain  = [...]byte{'I', 'S', 'O', 'C', '5', ':', ' ', 'd', 'o', 'm', 'a', 'i', 'n', ' ', 'o', 'k', '\n'}
	msgIsoC5Quota   = [...]byte{'I', 'S', 'O', 'C', '5', ':', ' ', 'q', 'u', 'o', 't', 'a', ' ', 'o', 'k', '\n'}
	msgIsoC5Cleanup = [...]byte{'I', 'S', 'O', 'C', '5', ':', ' ', 'c', 'l', 'e', 'a', 'n', 'u', 'p', ' ', 'o', 'k', '\n'}
	msgIsoC5Observe = [...]byte{'I', 'S', 'O', 'C', '5', ':', ' ', 'o', 'b', 's', 'e', 'r', 'v', 'e', ' ', 'o', 'k', '\n'}
	msgSoakC5Mixed  = [...]byte{'S', 'O', 'A', 'K', 'C', '5', ':', ' ', 'm', 'i', 'x', 'e', 'd', ' ', 'o', 'k', '\n'}

	pathCompatHello    = [...]byte{'/', 'c', 'o', 'm', 'p', 'a', 't', '/', 'h', 'e', 'l', 'l', 'o', '.', 't', 'x', 't', 0}
	pathRuntimeJournal = [...]byte{'/', 'r', 'u', 'n', 't', 'i', 'm', 'e', '/', 'j', 'o', 'u', 'r', 'n', 'a', 'l', '.', 'b', 'i', 'n', 0}
	pathRuntimeState   = [...]byte{'/', 'r', 'u', 'n', 't', 'i', 'm', 'e', '/', 's', 't', 'a', 't', 'e', '.', 'b', 'i', 'n', 0}

	c4RecoverSeed = [...]byte{'c', '4', '-', 'r', 'e', 'c', 'o', 'v', 'e', 'r'}
	c4FsyncSeed   = [...]byte{'c', '4', '-', 'f', 's', 'y', 'n', 'c'}
	c4TcpPing     = [...]byte{'t', 'c', 'p', '6', '-', 'o', 'k'}
	c5TcpPing     = [...]byte{'c', '5', '-', 's', 'o', 'a', 'k'}
)

func timeServiceMain() {
	log(msgTimeSvcStart[:])

	serviceEP := sysIpcEndpointCreate()
	if serviceEP == sysErr {
		setServiceState(serviceTime, stateFailed)
		fail(msgTimeSvcErr[:])
	}
	if sysSvcRegister(&nameTimeSvc[0], uintptr(len(nameTimeSvc)), serviceEP) == sysErr {
		setServiceState(serviceTime, stateFailed)
		fail(msgTimeSvcErr[:])
	}

	setServiceState(serviceTime, stateRunning)
	log(msgTimeSvcReady[:])

	for bootFailed == 0 {
		var req [8]byte
		n := sysIpcRecv(serviceEP, &req[0], uintptr(len(req)))
		if n == sysErr {
			setServiceState(serviceTime, stateFailed)
			fail(msgTimeSvcErr[:])
		}
		if n == 1 && req[0] == cmdStop {
			break
		}
		if n != 2 || req[0] != cmdTime {
			setServiceState(serviceTime, stateFailed)
			fail(msgTimeSvcErr[:])
		}
		log(msgTimeSvcReq[:])

		if sysTimeNow() == sysErr {
			setServiceState(serviceTime, stateFailed)
			fail(msgTimeSvcErr[:])
		}
		log(msgTimeSvcTime[:])

		if sysIpcSend(uintptr(req[1]), &replyOK[0], uintptr(len(replyOK))) == sysErr {
			setServiceState(serviceTime, stateFailed)
			fail(msgTimeSvcErr[:])
		}
	}

	setServiceState(serviceTime, stateStopping)
	setServiceState(serviceTime, stateStopped)
	sysThreadExit()
	fail(msgTimeSvcErr[:])
}

func diagServiceMain() {
	log(msgDiagSvcStart[:])

	serviceEP := sysIpcEndpointCreate()
	if serviceEP == sysErr {
		setServiceState(serviceDiag, stateFailed)
		fail(msgDiagSvcErr[:])
	}
	if sysSvcRegister(&nameDiagSvc[0], uintptr(len(nameDiagSvc)), serviceEP) == sysErr {
		setServiceState(serviceDiag, stateFailed)
		fail(msgDiagSvcErr[:])
	}

	setServiceState(serviceDiag, stateRunning)
	log(msgDiagSvcReady[:])

	for bootFailed == 0 {
		var req [8]byte
		n := sysIpcRecv(serviceEP, &req[0], uintptr(len(req)))
		if n == sysErr {
			setServiceState(serviceDiag, stateFailed)
			fail(msgDiagSvcErr[:])
		}
		if n == 1 && req[0] == cmdStop {
			log(msgDiagSvcStop[:])
			break
		}
		if n != 2 || req[0] != cmdDiag {
			setServiceState(serviceDiag, stateFailed)
			fail(msgDiagSvcErr[:])
		}

		log(msgDiagSvcSnap[:])
		logServiceSnapshot(serviceTime)
		logServiceSnapshot(serviceDiag)
		logServiceSnapshot(serviceShell)
		logServiceSnapshot(servicePkg)
		if !logKernelTaskSnapshot(serviceTime) {
			setServiceState(serviceDiag, stateFailed)
			fail(msgDiagSvcErr[:])
		}
		if !logKernelTaskSnapshot(serviceDiag) {
			setServiceState(serviceDiag, stateFailed)
			fail(msgDiagSvcErr[:])
		}
		if !logKernelTaskSnapshot(serviceShell) {
			setServiceState(serviceDiag, stateFailed)
			fail(msgDiagSvcErr[:])
		}
		if !logKernelTaskSnapshot(servicePkg) {
			setServiceState(serviceDiag, stateFailed)
			fail(msgDiagSvcErr[:])
		}

		if sysIpcSend(uintptr(req[1]), &replyOK[0], uintptr(len(replyOK))) == sysErr {
			setServiceState(serviceDiag, stateFailed)
			fail(msgDiagSvcErr[:])
		}
	}

	setServiceState(serviceDiag, stateStopping)
	setServiceState(serviceDiag, stateStopped)
	sysThreadExit()
	fail(msgDiagSvcErr[:])
}

func shellMain() {
	log(msgShellStart[:])

	if shellRecycles < 2 {
		shellRecycles++
		setServiceState(serviceShell, stateRunning)
		log(msgShellRecycle[:])
		sysThreadExit()
		fail(msgShellErr[:])
	}

	timeEP := sysSvcLookup(&nameTimeSvc[0], uintptr(len(nameTimeSvc)))
	if timeEP == sysErr {
		setServiceState(serviceShell, stateFailed)
		fail(msgShellErr[:])
	}
	log(msgShellLookup[:])

	var deny [1]byte
	if sysIpcRecv(timeEP, &deny[0], uintptr(len(deny))) != sysErr {
		setServiceState(serviceShell, stateFailed)
		fail(msgShellErr[:])
	}
	log(msgShellRecvDeny[:])

	if sysSvcRegister(&nameHijack[0], uintptr(len(nameHijack)), timeEP) != sysErr {
		setServiceState(serviceShell, stateFailed)
		fail(msgShellErr[:])
	}
	log(msgShellRegDeny[:])

	if sysThreadSpawn(spawnEntry) != sysErr {
		setServiceState(serviceShell, stateFailed)
		fail(msgShellErr[:])
	}
	log(msgShellSpawnDeny[:])

	replyEP := sysIpcEndpointCreate()
	if replyEP == sysErr {
		setServiceState(serviceShell, stateFailed)
		fail(msgShellErr[:])
	}

	setServiceState(serviceShell, stateRunning)

	if !requestTime(timeEP, replyEP) {
		setServiceState(serviceShell, stateFailed)
		fail(msgShellErr[:])
	}
	log(msgShellReply[:])

	diagEP := sysSvcLookup(&nameDiagSvc[0], uintptr(len(nameDiagSvc)))
	if diagEP == sysErr {
		setServiceState(serviceShell, stateFailed)
		fail(msgShellErr[:])
	}

	if !requestDiag(diagEP, replyEP) {
		setServiceState(serviceShell, stateFailed)
		fail(msgShellErr[:])
	}
	log(msgShellDiag[:])

	if !runC4Storage() || !runC4Network() {
		setServiceState(serviceShell, stateFailed)
		fail(msgShellErr[:])
	}
	if !runC5Isolation(replyEP, diagEP) || !runC5Reliability(replyEP, timeEP) {
		setServiceState(serviceShell, stateFailed)
		fail(msgShellErr[:])
	}
	if desktopProfileEnabled {
		if !runDesktopProfile(replyEP, diagEP) {
			setServiceState(serviceShell, stateFailed)
			fail(msgShellErr[:])
		}
	}

	pkgEP := sysSvcLookup(&namePkgSvc[0], uintptr(len(namePkgSvc)))
	if pkgEP == sysErr {
		setServiceState(serviceShell, stateFailed)
		fail(msgShellErr[:])
	}
	if pkgRuntimeAvailable() {
		if !requestPkg(pkgEP, replyEP) {
			setServiceState(serviceShell, stateFailed)
			fail(msgShellErr[:])
		}
		log(msgShellPkg[:])
	}

	setServiceState(serviceShell, stateStopping)
	shellComplete = 1
	setServiceState(serviceShell, stateStopped)
	sysThreadExit()
	fail(msgShellErr[:])
}

func pkgRuntimeAvailable() bool {
	fd := sysOpen(&pathPkgState[0], openReadWrite, 0)
	if fd == sysErr {
		return false
	}
	return sysClose(fd) != sysErr
}

func requestServiceCommand(serviceEP uintptr, replyEP uintptr, cmd byte) bool {
	req := [2]byte{cmd, byte(replyEP)}
	if sysIpcSend(serviceEP, &req[0], uintptr(len(req))) == sysErr {
		return false
	}

	var reply [4]byte
	n := sysIpcRecv(replyEP, &reply[0], uintptr(len(reply)))
	return n == uintptr(len(replyOK)) && reply[0] == replyOK[0] && reply[1] == replyOK[1]
}

func requestTime(serviceEP uintptr, replyEP uintptr) bool {
	return requestServiceCommand(serviceEP, replyEP, cmdTime)
}

func requestDiag(serviceEP uintptr, replyEP uintptr) bool {
	return requestServiceCommand(serviceEP, replyEP, cmdDiag)
}

func runC4Storage() bool {
	stateFD := sysOpen(&pathRuntimeState[0], openReadOnly, 0)
	if stateFD == sysErr {
		return true
	}

	var state [64]byte
	n := sysRead(stateFD, &state[0], uintptr(len(state)))
	if sysClose(stateFD) == sysErr || n == sysErr {
		return false
	}

	if n == 0 {
		journalFD := sysOpen(&pathRuntimeJournal[0], openWriteOnly, 0)
		if journalFD == sysErr {
			return false
		}
		if sysWrite(journalFD, &c4RecoverSeed[0], uintptr(len(c4RecoverSeed))) != uintptr(len(c4RecoverSeed)) {
			return false
		}
		if sysClose(journalFD) == sysErr {
			return false
		}
		log(msgStorC4Stage[:])
		return true
	}

	if bytesEqual(state[:], n, c4RecoverSeed[:]) {
		log(msgStorC4State[:])
		journalFD := sysOpen(&pathRuntimeJournal[0], openWriteOnly, 0)
		if journalFD == sysErr {
			return false
		}
		if sysWrite(journalFD, &c4FsyncSeed[0], uintptr(len(c4FsyncSeed))) != uintptr(len(c4FsyncSeed)) {
			return false
		}
		if sysFsync(journalFD) == sysErr || sysClose(journalFD) == sysErr {
			return false
		}
		stateFD = sysOpen(&pathRuntimeState[0], openReadOnly, 0)
		if stateFD == sysErr {
			return false
		}
		n = sysRead(stateFD, &state[0], uintptr(len(state)))
		if sysClose(stateFD) == sysErr || n == sysErr || !bytesEqual(state[:], n, c4FsyncSeed[:]) {
			return false
		}
		log(msgStorC4Fsync[:])
		return true
	}

	if bytesEqual(state[:], n, c4FsyncSeed[:]) {
		log(msgStorC4State[:])
		log(msgStorC4Fsync[:])
		return true
	}

	return false
}

func runC4Network() bool {
	return runSocketRoundTrip(4040, c4TcpPing[:], true)
}

func runC5Isolation(replyEP uintptr, diagEP uintptr) bool {
	if !verifyServiceProfiles(1) {
		return false
	}
	log(msgIsoC5Domain[:])

	if extraEP := sysIpcEndpointCreate(); extraEP == sysErr {
		return false
	} else {
		_ = extraEP
	}
	if sysIpcEndpointCreate() != sysErr {
		return false
	}

	fd0 := sysOpen(&pathCompatHello[0], openReadOnly, 0)
	if fd0 == sysErr {
		return false
	}
	fd1 := sysOpen(&pathCompatHello[0], openReadOnly, 0)
	if fd1 == sysErr {
		sysClose(fd0)
		return false
	}
	if fd2 := sysOpen(&pathCompatHello[0], openReadOnly, 0); fd2 != sysErr {
		sysClose(fd2)
		sysClose(fd1)
		sysClose(fd0)
		return false
	}
	if sysClose(fd1) == sysErr || sysClose(fd0) == sysErr {
		return false
	}

	s0 := sysSocketOpen(netFamilyInet6, socketStream)
	if s0 == sysErr {
		return false
	}
	s1 := sysSocketOpen(netFamilyInet6, socketStream)
	if s1 == sysErr {
		sysSocketClose(s0)
		return false
	}
	s2 := sysSocketOpen(netFamilyInet6, socketStream)
	if s2 == sysErr {
		sysSocketClose(s1)
		sysSocketClose(s0)
		return false
	}
	if s3 := sysSocketOpen(netFamilyInet6, socketStream); s3 != sysErr {
		sysSocketClose(s3)
		sysSocketClose(s2)
		sysSocketClose(s1)
		sysSocketClose(s0)
		return false
	}
	if sysSocketClose(s2) == sysErr || sysSocketClose(s1) == sysErr || sysSocketClose(s0) == sysErr {
		return false
	}

	if !verifyServiceProfiles(2) {
		return false
	}
	log(msgIsoC5Quota[:])

	if !requestDiag(diagEP, replyEP) {
		return false
	}
	log(msgIsoC5Observe[:])
	return true
}

func runC5Reliability(replyEP uintptr, timeEP uintptr) bool {
	var iter uintptr
	for iter = 0; iter < 3; iter++ {
		if !requestTime(timeEP, replyEP) {
			return false
		}
		if sysYield() != 0 {
			return false
		}
	}
	log(msgSoakC5Mixed[:])
	return true
}

func readRuntimeStateMaybe() bool {
	fd := sysOpen(&pathRuntimeState[0], openReadOnly, 0)
	if fd == sysErr {
		return true
	}
	var buf [64]byte
	n := sysRead(fd, &buf[0], uintptr(len(buf)))
	if sysClose(fd) == sysErr {
		return false
	}
	return n != sysErr
}

func verifyServiceProfiles(shellEndpointCount uint64) bool {
	var info taskInfo

	if !loadServiceTaskInfo(serviceTime, &info) {
		return false
	}
	if info.DomainID != 1 || info.CapabilityFlags != 0 || info.EndpointCount != 1 || info.FdCount != 0 || info.SocketCount != 0 {
		return false
	}

	if !loadServiceTaskInfo(serviceDiag, &info) {
		return false
	}
	if info.DomainID != 2 || info.CapabilityFlags != 0 || info.EndpointCount != 1 || info.FdCount != 0 || info.SocketCount != 0 {
		return false
	}

	if !loadServiceTaskInfo(serviceShell, &info) {
		return false
	}
	if info.DomainID != 3 {
		return false
	}
	if info.CapabilityFlags != uint64(taskCapStorage|taskCapNetwork) {
		return false
	}
	if info.EndpointCount != shellEndpointCount || info.FdCount != 0 || info.SocketCount != 0 {
		return false
	}

	if !loadServiceTaskInfo(servicePkg, &info) {
		return false
	}
	if info.DomainID != 4 {
		return false
	}
	if info.CapabilityFlags != uint64(taskCapStorage) {
		return false
	}
	if info.EndpointCount != 1 || info.FdCount != 0 || info.SocketCount != 0 {
		return false
	}

	return true
}

func loadServiceTaskInfo(serviceID uintptr, info *taskInfo) bool {
	if serviceTasks[serviceID] == taskUnset {
		return false
	}
	return sysProcInfo(uintptr(serviceTasks[serviceID]), info) != sysErr
}

func prepareRuntimeListenAddr(port uint64, logConfig bool) (socketAddr, bool) {
	var ifcfg netAddrConfig
	ifcfg.Family = netFamilyInet6
	ifcfg.PrefixLen = 64
	ifcfg.Addr = [16]byte{0xfd, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2}

	ifIndex := uintptr(1)
	if sysNetIfConfig(ifIndex, &ifcfg) == sysErr {
		ifIndex = 0
		ifcfg.PrefixLen = 128
		ifcfg.Addr = [16]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1}
		if sysNetIfConfig(ifIndex, &ifcfg) == sysErr {
			return socketAddr{}, false
		}
	}
	if logConfig {
		log(msgNetC4IfCfg[:])
	}

	if sysNetRouteAdd(ifIndex, &ifcfg) == sysErr {
		return socketAddr{}, false
	}
	if logConfig {
		log(msgNetC4Route[:])
	}

	return socketAddr{
		Family: netFamilyInet6,
		Port:   port,
		Addr:   ifcfg.Addr,
	}, true
}

func runSocketLifecycle(port uint64) bool {
	listenAddr, ok := prepareRuntimeListenAddr(port, false)
	if !ok {
		return false
	}

	server := sysSocketOpen(netFamilyInet6, socketStream)
	if server == sysErr {
		return false
	}
	if sysSocketBind(server, &listenAddr) == sysErr {
		sysSocketClose(server)
		return false
	}
	if sysSocketListen(server, 1) == sysErr {
		sysSocketClose(server)
		return false
	}
	return sysSocketClose(server) != sysErr
}

func runSocketRoundTrip(port uint64, payload []byte, logLifecycle bool) bool {
	listenAddr, ok := prepareRuntimeListenAddr(port, logLifecycle)
	if !ok {
		return false
	}

	server := sysSocketOpen(netFamilyInet6, socketStream)
	if server == sysErr {
		return false
	}
	if sysSocketBind(server, &listenAddr) == sysErr {
		sysSocketClose(server)
		return false
	}
	if sysSocketListen(server, 1) == sysErr {
		sysSocketClose(server)
		return false
	}
	if logLifecycle {
		log(msgNetC4Listen[:])
	}

	client := sysSocketOpen(netFamilyInet6, socketStream)
	if client == sysErr {
		sysSocketClose(server)
		return false
	}
	if sysSocketConnect(client, &listenAddr) == sysErr {
		sysSocketClose(client)
		sysSocketClose(server)
		return false
	}
	if logLifecycle {
		log(msgNetC4Connect[:])
	}

	var peerAddr socketAddr
	addrLen := uintptr(32)
	accepted := sysSocketAccept(server, &peerAddr, &addrLen)
	if accepted == sysErr {
		sysSocketClose(client)
		sysSocketClose(server)
		return false
	}
	if logLifecycle {
		log(msgNetC4Accept[:])
	}

	if sysSocketSend(client, &payload[0], uintptr(len(payload))) != uintptr(len(payload)) {
		sysSocketClose(accepted)
		sysSocketClose(client)
		sysSocketClose(server)
		return false
	}

	var recvBuf [16]byte
	n := sysSocketRecv(accepted, &recvBuf[0], uintptr(len(recvBuf)))
	if !bytesEqual(recvBuf[:], n, payload) {
		sysSocketClose(accepted)
		sysSocketClose(client)
		sysSocketClose(server)
		return false
	}
	if logLifecycle {
		log(msgNetC4Recv[:])
	}

	if sysSocketSend(accepted, &replyOK[0], uintptr(len(replyOK))) != uintptr(len(replyOK)) {
		sysSocketClose(accepted)
		sysSocketClose(client)
		sysSocketClose(server)
		return false
	}
	n = sysSocketRecv(client, &recvBuf[0], uintptr(len(recvBuf)))
	if !bytesEqual(recvBuf[:], n, replyOK[:]) {
		sysSocketClose(accepted)
		sysSocketClose(client)
		sysSocketClose(server)
		return false
	}
	if logLifecycle {
		log(msgNetC4Reply[:])
	}

	if sysSocketClose(accepted) == sysErr || sysSocketClose(client) == sysErr || sysSocketClose(server) == sysErr {
		return false
	}
	return true
}

func bytesEqual(buf []byte, n uintptr, expect []byte) bool {
	if n == sysErr || int(n) != len(expect) {
		return false
	}
	var idx uintptr
	for idx = 0; idx < n; idx++ {
		if buf[idx] != expect[idx] {
			return false
		}
	}
	return true
}
