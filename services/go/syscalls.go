package main

import "unsafe"

const (
	taskStateReady = iota
	taskStateRunning
	taskStateBlocked
	taskStateExited
	taskStateDead
)

const (
	schedClassBestEffort = iota
	schedClassCritical
)

const (
	openReadOnly = iota
	openWriteOnly
	openReadWrite
)

// openCreate ORs into the open flags: create the /data file when absent.
const openCreate = 0x4

const (
	fsCtlMkdir = 1
	fsCtlUnlink = 2
	fsCtlStat = 3
	fsCtlPipe = 4
	fsCtlChmod = 5
)

const (
	netFamilyInet  = 2
	netFamilyInet6 = 10
)

const (
	socketStream = 1
)

const (
	taskCapStorage = 1 << iota
	taskCapNetwork
)

type taskInfo struct {
	TID             uint64
	ParentTID       uint64
	State           uint64
	SchedClass      uint64
	DispatchCount   uint64
	YieldCount      uint64
	BlockCount      uint64
	IpcSendCount    uint64
	IpcRecvCount    uint64
	EndpointCount   uint64
	ShmCount        uint64
	ThreadCount     uint64
	ExitStatus      uint64
	DomainID        uint64
	CapabilityFlags uint64
	FdCount         uint64
	SocketCount     uint64
}

type socketAddr struct {
	Family uint64
	Port   uint64
	Addr   [16]byte
}

type netAddrConfig struct {
	Family    uint64
	PrefixLen uint64
	Addr      [16]byte
}

type isolationConfig struct {
	DomainID        uint64
	CapabilityFlags uint64
	Limits          uint64
}

// sysDebugWrite invokes syscall 0 (sys_debug_write).
func sysDebugWrite(buf *byte, n uintptr) uintptr

// sysThreadSpawn invokes syscall 1 (sys_thread_spawn).
func sysThreadSpawn(entry uintptr) uintptr

// sysThreadExit invokes syscall 2 (sys_thread_exit).
func sysThreadExit() uintptr

// sysYield invokes syscall 3 (sys_yield).
func sysYield() uintptr

// sysIpcSend invokes syscall 8 (sys_ipc_send).
func sysIpcSend(ep uintptr, buf *byte, n uintptr) uintptr

// sysIpcRecv invokes syscall 9 (sys_ipc_recv).
func sysIpcRecv(ep uintptr, buf *byte, cap uintptr) uintptr

// sysTimeNow invokes syscall 10 (sys_time_now).
func sysTimeNow() uintptr

// sysWait invokes syscall 22 (sys_wait).
func sysWait(pid uintptr, status *uintptr, options uintptr) uintptr

// sysSpawn invokes syscall 46 (sys_spawn): run a named app from the
// package store on disk as a child task.
func sysSpawn(name *byte, n uintptr, args *byte, alen uintptr, stdinFd uintptr, stdoutFd uintptr) uintptr

// sysFsCtl invokes syscall 47 (sys_fs_ctl): mkdir/unlink/stat on /data.
func sysFsCtl(op uintptr, path *byte, arg uintptr) uintptr

// sysOpenRaw invokes syscall 18 (sys_open).
func sysOpenRaw(path *byte, flags uintptr, mode uintptr) uintptr

// sysReadRaw invokes syscall 19 (sys_read).
func sysReadRaw(fd uintptr, buf *byte, n uintptr) uintptr

// sysWriteRaw invokes syscall 20 (sys_write).
func sysWriteRaw(fd uintptr, buf *byte, n uintptr) uintptr

// sysCloseRaw invokes syscall 21 (sys_close).
func sysCloseRaw(fd uintptr) uintptr

// sysSvcRegister invokes syscall 11 (sys_svc_register).
func sysSvcRegister(name *byte, n uintptr, ep uintptr) uintptr

// sysSvcLookup invokes syscall 12 (sys_svc_lookup).
func sysSvcLookup(name *byte, n uintptr) uintptr

// sysIpcEndpointCreate invokes syscall 17 (sys_ipc_endpoint_create).
func sysIpcEndpointCreate() uintptr

// sysSchedSetRaw invokes syscall 29 (sys_sched_set).
func sysSchedSetRaw(tid uintptr, class uintptr) uintptr

// sysProcInfoRaw invokes syscall 28 (sys_proc_info).
func sysProcInfoRaw(tid uintptr, buf *byte, n uintptr) uintptr

// sysFsyncRaw invokes syscall 30 (sys_fsync).
func sysFsyncRaw(fd uintptr) uintptr

// sysSocketOpenRaw invokes syscall 31 (sys_socket_open).
func sysSocketOpenRaw(domain uintptr, kind uintptr) uintptr

// sysSocketBindRaw invokes syscall 32 (sys_socket_bind).
func sysSocketBindRaw(socket uintptr, addr *byte, n uintptr) uintptr

// sysSocketListenRaw invokes syscall 33 (sys_socket_listen).
func sysSocketListenRaw(socket uintptr, backlog uintptr) uintptr

// sysSocketConnectRaw invokes syscall 34 (sys_socket_connect).
func sysSocketConnectRaw(socket uintptr, addr *byte, n uintptr) uintptr

// sysSocketAcceptRaw invokes syscall 35 (sys_socket_accept).
func sysSocketAcceptRaw(socket uintptr, addr *byte, addrLen *uintptr) uintptr

// sysSocketSendRaw invokes syscall 36 (sys_socket_send).
func sysSocketSendRaw(socket uintptr, buf *byte, n uintptr) uintptr

// sysSocketRecvRaw invokes syscall 37 (sys_socket_recv).
func sysSocketRecvRaw(socket uintptr, buf *byte, n uintptr) uintptr

// sysSocketCloseRaw invokes syscall 38 (sys_socket_close).
func sysSocketCloseRaw(socket uintptr) uintptr

// sysNetIfConfigRaw invokes syscall 39 (sys_net_if_config).
func sysNetIfConfigRaw(ifIndex uintptr, cfg *byte, n uintptr) uintptr

// sysNetRouteAddRaw invokes syscall 40 (sys_net_route_add).
func sysNetRouteAddRaw(ifIndex uintptr, cfg *byte, n uintptr) uintptr

// sysIsolationConfigRaw invokes syscall 41 (sys_isolation_config).
func sysIsolationConfigRaw(tid uintptr, cfg *byte, n uintptr) uintptr

func sysSchedSet(tid uintptr, class uintptr) uintptr {
	return sysSchedSetRaw(tid, class)
}

func sysOpen(path *byte, flags uintptr, mode uintptr) uintptr {
	return sysOpenRaw(path, flags, mode)
}

func sysRead(fd uintptr, buf *byte, n uintptr) uintptr {
	return sysReadRaw(fd, buf, n)
}

func sysWrite(fd uintptr, buf *byte, n uintptr) uintptr {
	return sysWriteRaw(fd, buf, n)
}

func sysClose(fd uintptr) uintptr {
	return sysCloseRaw(fd)
}

func sysProcInfo(tid uintptr, info *taskInfo) uintptr {
	return sysProcInfoRaw(
		tid,
		(*byte)(unsafe.Pointer(info)),
		unsafe.Sizeof(*info),
	)
}

func sysFsync(fd uintptr) uintptr {
	return sysFsyncRaw(fd)
}

func sysSocketOpen(domain uintptr, kind uintptr) uintptr {
	return sysSocketOpenRaw(domain, kind)
}

func sysSocketBind(socket uintptr, addr *socketAddr) uintptr {
	return sysSocketBindRaw(
		socket,
		(*byte)(unsafe.Pointer(addr)),
		unsafe.Sizeof(*addr),
	)
}

func sysSocketListen(socket uintptr, backlog uintptr) uintptr {
	return sysSocketListenRaw(socket, backlog)
}

func sysSocketConnect(socket uintptr, addr *socketAddr) uintptr {
	return sysSocketConnectRaw(
		socket,
		(*byte)(unsafe.Pointer(addr)),
		unsafe.Sizeof(*addr),
	)
}

func sysSocketAccept(socket uintptr, addr *socketAddr, addrLen *uintptr) uintptr {
	return sysSocketAcceptRaw(
		socket,
		(*byte)(unsafe.Pointer(addr)),
		addrLen,
	)
}

func sysSocketSend(socket uintptr, buf *byte, n uintptr) uintptr {
	return sysSocketSendRaw(socket, buf, n)
}

func sysSocketRecv(socket uintptr, buf *byte, n uintptr) uintptr {
	return sysSocketRecvRaw(socket, buf, n)
}

func sysSocketClose(socket uintptr) uintptr {
	return sysSocketCloseRaw(socket)
}

func sysNetIfConfig(ifIndex uintptr, cfg *netAddrConfig) uintptr {
	return sysNetIfConfigRaw(
		ifIndex,
		(*byte)(unsafe.Pointer(cfg)),
		unsafe.Sizeof(*cfg),
	)
}

func sysNetRouteAdd(ifIndex uintptr, cfg *netAddrConfig) uintptr {
	return sysNetRouteAddRaw(
		ifIndex,
		(*byte)(unsafe.Pointer(cfg)),
		unsafe.Sizeof(*cfg),
	)
}

func packIsolationLimits(fdLimit byte, socketLimit byte, endpointLimit byte) uint64 {
	return uint64(fdLimit) | (uint64(socketLimit) << 8) | (uint64(endpointLimit) << 16)
}

func sysIsolationConfig(tid uintptr, cfg *isolationConfig) uintptr {
	return sysIsolationConfigRaw(
		tid,
		(*byte)(unsafe.Pointer(cfg)),
		unsafe.Sizeof(*cfg),
	)
}

// sysSpawnEntry returns the user-mode trampoline for spawned threads.
func sysSpawnEntry() uintptr

// haltForever never returns and is implemented in start.asm.
func haltForever()
