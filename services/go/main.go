package main

const (
	sysErr      = ^uintptr(0)
	roleSvcMgr  = 0
	roleTimeSvc = 1
	roleShell   = 2
	cmdTime     = 'T'
)

var (
	spawnOrdinal  uintptr
	bootFailed    uintptr
	timesvcReady  uintptr
	shellComplete uintptr
)

var msgGoInitStart = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 's', 't', 'a', 'r', 't', '\n'}
var msgGoInitSpawn = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 's', 'v', 'c', 'm', 'g', 'r', ' ', 'u', 'p', '\n'}
var msgGoInitReady = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 'r', 'e', 'a', 'd', 'y', '\n'}
var msgGoInitErr = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 'e', 'r', 'r', '\n'}

var msgSvcMgrStart = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 's', 't', 'a', 'r', 't', '\n'}
var msgSvcMgrShell = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 's', 'h', 'e', 'l', 'l', '\n'}
var msgSvcMgrErr = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 'e', 'r', 'r', '\n'}

var msgTimeSvcStart = [...]byte{'T', 'I', 'M', 'E', 'S', 'V', 'C', ':', ' ', 's', 't', 'a', 'r', 't', '\n'}
var msgTimeSvcReady = [...]byte{'T', 'I', 'M', 'E', 'S', 'V', 'C', ':', ' ', 'r', 'e', 'a', 'd', 'y', '\n'}
var msgTimeSvcReq = [...]byte{'T', 'I', 'M', 'E', 'S', 'V', 'C', ':', ' ', 'r', 'e', 'q', ' ', 'o', 'k', '\n'}
var msgTimeSvcTime = [...]byte{'T', 'I', 'M', 'E', 'S', 'V', 'C', ':', ' ', 't', 'i', 'm', 'e', ' ', 'o', 'k', '\n'}
var msgTimeSvcErr = [...]byte{'T', 'I', 'M', 'E', 'S', 'V', 'C', ':', ' ', 'e', 'r', 'r', '\n'}

var msgShellStart = [...]byte{'G', 'O', 'S', 'H', ':', ' ', 's', 't', 'a', 'r', 't', '\n'}
var msgShellLookup = [...]byte{'G', 'O', 'S', 'H', ':', ' ', 'l', 'o', 'o', 'k', 'u', 'p', ' ', 'o', 'k', '\n'}
var msgShellReply = [...]byte{'G', 'O', 'S', 'H', ':', ' ', 'r', 'e', 'p', 'l', 'y', ' ', 'o', 'k', '\n'}
var msgShellErr = [...]byte{'G', 'O', 'S', 'H', ':', ' ', 'e', 'r', 'r', '\n'}

var serviceName = [...]byte{'t', 'i', 'm', 'e', 's', 'v', 'c'}
var replyOK = [...]byte{'O', 'K'}

func main() {
	log(msgGoInitStart[:])

	entry := sysSpawnEntry()
	if entry == 0 || sysThreadSpawn(entry) == sysErr {
		fail(msgGoInitErr[:])
	}
	log(msgGoInitSpawn[:])

	for shellComplete == 0 && bootFailed == 0 {
		if sysYield() != 0 {
			fail(msgGoInitErr[:])
		}
	}
	if bootFailed != 0 {
		sysThreadExit()
		fail(msgGoInitErr[:])
	}

	log(msgGoInitReady[:])
	sysThreadExit()
	fail(msgGoInitErr[:])
}

//export goSpawnedThreadMain
func spawnedThreadMain() {
	role := spawnOrdinal
	spawnOrdinal = role + 1

	switch role {
	case roleSvcMgr:
		serviceManagerMain()
	case roleTimeSvc:
		timeServiceMain()
	case roleShell:
		shellMain()
	default:
		fail(msgGoInitErr[:])
	}
}

func serviceManagerMain() {
	log(msgSvcMgrStart[:])

	entry := sysSpawnEntry()
	if entry == 0 || sysThreadSpawn(entry) == sysErr {
		fail(msgSvcMgrErr[:])
	}

	for timesvcReady == 0 && bootFailed == 0 {
		if sysYield() != 0 {
			fail(msgSvcMgrErr[:])
		}
	}
	if bootFailed != 0 {
		sysThreadExit()
		fail(msgSvcMgrErr[:])
	}

	if sysThreadSpawn(entry) == sysErr {
		fail(msgSvcMgrErr[:])
	}

	log(msgSvcMgrShell[:])
	sysThreadExit()
	fail(msgSvcMgrErr[:])
}

func timeServiceMain() {
	log(msgTimeSvcStart[:])

	serviceEP := sysIpcEndpointCreate()
	if serviceEP == sysErr {
		fail(msgTimeSvcErr[:])
	}
	if sysSvcRegister(&serviceName[0], uintptr(len(serviceName)), serviceEP) == sysErr {
		fail(msgTimeSvcErr[:])
	}

	timesvcReady = 1
	log(msgTimeSvcReady[:])

	var req [8]byte
	n := sysIpcRecv(serviceEP, &req[0], uintptr(len(req)))
	if n != 2 || req[0] != cmdTime {
		fail(msgTimeSvcErr[:])
	}
	log(msgTimeSvcReq[:])

	if sysTimeNow() == sysErr {
		fail(msgTimeSvcErr[:])
	}
	log(msgTimeSvcTime[:])

	if sysIpcSend(uintptr(req[1]), &replyOK[0], uintptr(len(replyOK))) == sysErr {
		fail(msgTimeSvcErr[:])
	}

	sysThreadExit()
	fail(msgTimeSvcErr[:])
}

func shellMain() {
	log(msgShellStart[:])

	serviceEP := sysSvcLookup(&serviceName[0], uintptr(len(serviceName)))
	if serviceEP == sysErr {
		fail(msgShellErr[:])
	}
	log(msgShellLookup[:])

	replyEP := sysIpcEndpointCreate()
	if replyEP == sysErr {
		fail(msgShellErr[:])
	}

	req := [2]byte{cmdTime, byte(replyEP)}
	if sysIpcSend(serviceEP, &req[0], uintptr(len(req))) == sysErr {
		fail(msgShellErr[:])
	}

	var reply [4]byte
	n := sysIpcRecv(replyEP, &reply[0], uintptr(len(reply)))
	if n != uintptr(len(replyOK)) || reply[0] != replyOK[0] || reply[1] != replyOK[1] {
		fail(msgShellErr[:])
	}
	log(msgShellReply[:])

	shellComplete = 1
	sysThreadExit()
	fail(msgShellErr[:])
}

func log(msg []byte) {
	sysDebugWrite(&msg[0], uintptr(len(msg)))
}

func fail(msg []byte) {
	bootFailed = 1
	log(msg)
	sysThreadExit()
	haltForever()
}

// sysDebugWrite invokes syscall 0 (sys_debug_write).
func sysDebugWrite(buf *byte, n uintptr) uintptr

// sysThreadSpawn invokes syscall 1 (sys_thread_spawn).
func sysThreadSpawn(entry uintptr) uintptr

// sysThreadExit invokes syscall 2 (sys_thread_exit).
func sysThreadExit() uintptr

// sysYield invokes syscall 3 (sys_yield).
func sysYield() uintptr

// sysIpcEndpointCreate invokes syscall 17 (sys_ipc_endpoint_create).
func sysIpcEndpointCreate() uintptr

// sysIpcSend invokes syscall 8 (sys_ipc_send).
func sysIpcSend(ep uintptr, buf *byte, n uintptr) uintptr

// sysIpcRecv invokes syscall 9 (sys_ipc_recv).
func sysIpcRecv(ep uintptr, buf *byte, cap uintptr) uintptr

// sysTimeNow invokes syscall 10 (sys_time_now).
func sysTimeNow() uintptr

// sysSvcRegister invokes syscall 11 (sys_svc_register).
func sysSvcRegister(name *byte, n uintptr, ep uintptr) uintptr

// sysSvcLookup invokes syscall 12 (sys_svc_lookup).
func sysSvcLookup(name *byte, n uintptr) uintptr

// sysSpawnEntry returns the user-mode trampoline for spawned threads.
func sysSpawnEntry() uintptr

// haltForever never returns and is implemented in start.asm.
func haltForever()
