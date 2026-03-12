package main

const (
	sysErr  = ^uintptr(0)
	waitAny = ^uintptr(0)
	cmdTime = 'T'
	cmdDiag = 'D'
	cmdStop = 'Q'
)

const (
	serviceTime = iota
	serviceDiag
	serviceShell
	serviceCount
)

const (
	classCritical = iota
	classBestEffort
)

const (
	restartNever = iota
	restartOnFailure
	restartAlways
)

const (
	phaseCore = iota
	phaseServices
)

const (
	stateDeclared = iota
	stateBlocked
	stateStarting
	stateRunning
	stateFailed
	stateStopping
	stateStopped
)

const maxServiceRestarts = 3
const stateUnset = 0xFF
const taskUnset = 0xFF

type serviceSpec struct {
	name        []byte
	class       byte
	policy      byte
	deps        byte
	phase       byte
	startBudget byte
	stopCmd     byte
}

var (
	spawnEntry      uintptr
	spawnServiceID  uintptr
	bootFailed      uintptr
	bootOperational uintptr
	shutdownStarted uintptr
	shellComplete   uintptr

	serviceStates = [serviceCount]byte{stateUnset, stateUnset, stateUnset}
	serviceTasks  = [serviceCount]byte{taskUnset, taskUnset, taskUnset}

	serviceRestarts [serviceCount]byte
	serviceStarts   [serviceCount]byte
	serviceFailures [serviceCount]byte
	serviceReaps    [serviceCount]byte
	serviceLastTick [serviceCount]uintptr
	shellRecycles   byte
)

var (
	nameTimeSvc = [...]byte{'t', 'i', 'm', 'e', 's', 'v', 'c'}
	nameDiagSvc = [...]byte{'d', 'i', 'a', 'g', 's', 'v', 'c'}
	nameShell   = [...]byte{'s', 'h', 'e', 'l', 'l'}
	nameHijack  = [...]byte{'h', 'i', 'j', 'a', 'c', 'k'}
	replyOK     = [...]byte{'O', 'K'}
)

var serviceManifest = [...]serviceSpec{
	{
		name:        nameTimeSvc[:],
		class:       classCritical,
		policy:      restartOnFailure,
		deps:        0,
		phase:       phaseCore,
		startBudget: 8,
		stopCmd:     cmdStop,
	},
	{
		name:        nameDiagSvc[:],
		class:       classBestEffort,
		policy:      restartOnFailure,
		deps:        1 << serviceTime,
		phase:       phaseServices,
		startBudget: 8,
		stopCmd:     cmdStop,
	},
	{
		name:        nameShell[:],
		class:       classBestEffort,
		policy:      restartOnFailure,
		deps:        1 << serviceTime,
		phase:       phaseServices,
		startBudget: 8,
		stopCmd:     0,
	},
}

var restartBackoffYields = [...]uintptr{1, 2, 4}

var (
	msgGoInitStart       = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 's', 't', 'a', 'r', 't', '\n'}
	msgGoInitBootstrap   = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 'b', 'o', 'o', 't', 's', 't', 'r', 'a', 'p', '\n'}
	msgGoInitSpawn       = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 's', 'v', 'c', 'm', 'g', 'r', ' ', 'u', 'p', '\n'}
	msgGoInitOperational = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 'o', 'p', 'e', 'r', 'a', 't', 'i', 'o', 'n', 'a', 'l', '\n'}
	msgGoInitReady       = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 'r', 'e', 'a', 'd', 'y', '\n'}
	msgGoInitErr         = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 'e', 'r', 'r', '\n'}

	msgSvcMgrStart = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 's', 't', 'a', 'r', 't', '\n'}
	msgSvcMgrShell = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 's', 'h', 'e', 'l', 'l', '\n'}
	msgSvcMgrReap  = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 'r', 'e', 'a', 'p', ' '}
	msgSvcMgrRetry = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 'r', 'e', 's', 't', 'a', 'r', 't', ' '}
	msgSvcMgrStop  = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 's', 't', 'o', 'p', ' '}
	msgSvcMgrWedge = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 'w', 'e', 'd', 'g', 'e', ' '}
	msgSvcMgrErr   = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 'e', 'r', 'r', '\n'}

	msgServicePrefix = [...]byte{'S', 'V', 'C', ':', ' '}
	msgProcPrefix    = [...]byte{'P', 'R', 'O', 'C', ':', ' '}
	msgMetricStarts  = [...]byte{'s', '='}
	msgMetricRest    = [...]byte{'r', '='}
	msgMetricFail    = [...]byte{'f', '='}
	msgMetricReaps   = [...]byte{'x', '='}
	msgMetricTick    = [...]byte{'t', '='}
	msgSpace         = [...]byte{' '}
	msgNewline       = [...]byte{'\n'}

	msgStateDeclared = [...]byte{'d', 'e', 'c', 'l', 'a', 'r', 'e', 'd'}
	msgStateBlocked  = [...]byte{'b', 'l', 'o', 'c', 'k', 'e', 'd'}
	msgStateStarting = [...]byte{'s', 't', 'a', 'r', 't', 'i', 'n', 'g'}
	msgStateRunning  = [...]byte{'r', 'u', 'n', 'n', 'i', 'n', 'g'}
	msgStateFailed   = [...]byte{'f', 'a', 'i', 'l', 'e', 'd'}
	msgStateStopping = [...]byte{'s', 't', 'o', 'p', 'p', 'i', 'n', 'g'}
	msgStateStopped  = [...]byte{'s', 't', 'o', 'p', 'p', 'e', 'd'}
)

func bootRuntime() {
	log(msgGoInitStart[:])

	spawnEntry = sysSpawnEntry()
	if spawnEntry == 0 {
		fail(msgGoInitErr[:])
	}

	log(msgGoInitBootstrap[:])

	var order [serviceCount]byte
	if !buildStartPlan(&order) {
		fail(msgGoInitErr[:])
	}

	log(msgGoInitSpawn[:])
	serviceManagerMain(order)
}

func serviceManagerMain(order [serviceCount]byte) {
	log(msgSvcMgrStart[:])

	var idx uintptr
	for idx = 0; idx < serviceCount; idx++ {
		setServiceState(uintptr(order[idx]), stateDeclared)
	}

	liveChildren := launchPhase(order, phaseCore)
	liveChildren += launchPhase(order, phaseServices)

	for liveChildren != 0 && bootFailed == 0 {
		serviceID, restart, ok := reapService()
		if !ok {
			fail(msgSvcMgrErr[:])
		}

		if restart {
			logServiceAction(msgSvcMgrRetry[:], serviceID)
			if !scheduleRestart(serviceID) {
				if serviceNeedsSuccess(serviceID) {
					fail(msgSvcMgrErr[:])
				}
				liveChildren--
				continue
			}
			if !launchService(serviceID) {
				if serviceNeedsSuccess(serviceID) {
					fail(msgSvcMgrErr[:])
				}
				liveChildren--
			}
			continue
		}

		liveChildren--
		if shellComplete != 0 && shutdownStarted == 0 {
			shutdownStarted = 1
			if !beginShutdown(order) {
				fail(msgSvcMgrErr[:])
			}
		}
	}

	if bootFailed != 0 {
		fail(msgSvcMgrErr[:])
	}

	if serviceStates[serviceShell] != stateStopped {
		fail(msgSvcMgrErr[:])
	}
	if serviceStates[serviceTime] != stateStopped {
		fail(msgSvcMgrErr[:])
	}
	if serviceStates[serviceDiag] != stateStopped {
		fail(msgSvcMgrErr[:])
	}

	log(msgGoInitReady[:])
	sysThreadExit()
	fail(msgSvcMgrErr[:])
}

func launchPhase(order [serviceCount]byte, phase byte) uintptr {
	var idx uintptr
	var liveChildren uintptr

	for idx = 0; idx < serviceCount; idx++ {
		serviceID := uintptr(order[idx])
		spec := serviceManifest[serviceID]
		if spec.phase != phase {
			continue
		}
		if !depsRunning(spec.deps) {
			setServiceState(serviceID, stateBlocked)
			if spec.class == classCritical {
				fail(msgSvcMgrErr[:])
			}
			continue
		}

		if serviceID == serviceShell {
			log(msgSvcMgrShell[:])
		}

		if !launchService(serviceID) {
			if serviceNeedsSuccess(serviceID) {
				fail(msgSvcMgrErr[:])
			}
			continue
		}
		liveChildren++

		if bootOperational == 0 && allCriticalRunning() {
			bootOperational = 1
			log(msgGoInitOperational[:])
		}
	}

	return liveChildren
}

func buildStartPlan(order *[serviceCount]byte) bool {
	var pending byte
	var nextSlot uintptr
	var serviceID uintptr

	for serviceID = 0; serviceID < serviceCount; serviceID++ {
		pending |= 1 << serviceID
		if serviceManifest[serviceID].deps&(1<<serviceID) != 0 {
			return false
		}
		if serviceManifest[serviceID].deps&^pendingMask() != 0 {
			return false
		}
	}

	for nextSlot = 0; nextSlot < serviceCount; nextSlot++ {
		chosen := uintptr(serviceCount)
		for serviceID = 0; serviceID < serviceCount; serviceID++ {
			if pending&(1<<serviceID) == 0 {
				continue
			}
			if serviceManifest[serviceID].deps&pending != 0 {
				continue
			}
			if chosen == serviceCount || compareNames(serviceManifest[serviceID].name, serviceManifest[chosen].name) < 0 {
				chosen = serviceID
			}
		}

		if chosen == serviceCount {
			return false
		}

		order[nextSlot] = byte(chosen)
		pending &^= 1 << chosen
	}

	return true
}

func pendingMask() byte {
	var mask byte
	var serviceID uintptr
	for serviceID = 0; serviceID < serviceCount; serviceID++ {
		mask |= 1 << serviceID
	}
	return mask
}

func compareNames(left []byte, right []byte) int {
	var idx uintptr
	limit := uintptr(len(left))
	if uintptr(len(right)) < limit {
		limit = uintptr(len(right))
	}
	for idx = 0; idx < limit; idx++ {
		if left[idx] < right[idx] {
			return -1
		}
		if left[idx] > right[idx] {
			return 1
		}
	}
	if len(left) < len(right) {
		return -1
	}
	if len(left) > len(right) {
		return 1
	}
	return 0
}

func depsRunning(mask byte) bool {
	var serviceID uintptr
	for serviceID = 0; serviceID < serviceCount; serviceID++ {
		if mask&(1<<serviceID) == 0 {
			continue
		}
		if serviceStates[serviceID] != stateRunning {
			return false
		}
	}
	return true
}

func allCriticalRunning() bool {
	var serviceID uintptr
	for serviceID = 0; serviceID < serviceCount; serviceID++ {
		if serviceManifest[serviceID].class != classCritical {
			continue
		}
		if serviceStates[serviceID] != stateRunning {
			return false
		}
	}
	return true
}

func launchService(serviceID uintptr) bool {
	for {
		setServiceState(serviceID, stateStarting)
		spawnServiceID = serviceID
		tid := sysThreadSpawn(spawnEntry)
		if tid == sysErr {
			setServiceState(serviceID, stateFailed)
			serviceTasks[serviceID] = taskUnset
		} else {
			serviceTasks[serviceID] = byte(tid)
		}

		budget := uintptr(serviceManifest[serviceID].startBudget)
		for serviceStates[serviceID] == stateStarting && bootFailed == 0 {
			if budget == 0 {
				logServiceAction(msgSvcMgrWedge[:], serviceID)
				setServiceState(serviceID, stateFailed)
				break
			}
			if sysYield() != 0 {
				fail(msgSvcMgrErr[:])
			}
			budget--
		}

		switch serviceStates[serviceID] {
		case stateRunning:
			return true
		case stateFailed:
			if !scheduleRestart(serviceID) {
				return false
			}
		default:
			return false
		}
	}
}

func reapService() (uintptr, bool, bool) {
	var status uintptr
	tid := sysWait(waitAny, &status, 0)
	_ = status
	if tid == sysErr {
		return serviceCount, false, false
	}

	serviceID := serviceByTask(tid)
	if serviceID == serviceCount {
		return serviceCount, false, false
	}

	serviceTasks[serviceID] = taskUnset
	serviceReaps[serviceID]++
	logServiceAction(msgSvcMgrReap[:], serviceID)

	switch serviceStates[serviceID] {
	case stateStopped:
		return serviceID, false, true
	case stateFailed:
		return serviceID, true, true
	default:
		setServiceState(serviceID, stateFailed)
		return serviceID, true, true
	}
}

func serviceByTask(tid uintptr) uintptr {
	var serviceID uintptr
	for serviceID = 0; serviceID < serviceCount; serviceID++ {
		if serviceTasks[serviceID] == byte(tid) {
			return serviceID
		}
	}
	return serviceCount
}

func serviceNeedsSuccess(serviceID uintptr) bool {
	return serviceManifest[serviceID].class == classCritical || serviceID == serviceShell
}

func scheduleRestart(serviceID uintptr) bool {
	policy := serviceManifest[serviceID].policy
	if policy == restartNever {
		return false
	}
	if serviceRestarts[serviceID] >= maxServiceRestarts {
		return false
	}

	backoff := restartBackoffYields[serviceRestarts[serviceID]]
	serviceRestarts[serviceID]++
	return yieldCount(backoff)
}

func yieldCount(count uintptr) bool {
	var idx uintptr
	for idx = 0; idx < count; idx++ {
		if sysYield() != 0 {
			return false
		}
	}
	return true
}

func beginShutdown(order [serviceCount]byte) bool {
	var idx uintptr = serviceCount
	for idx > 0 {
		idx--
		serviceID := uintptr(order[idx])
		if serviceID == serviceShell {
			continue
		}
		if serviceStates[serviceID] != stateRunning {
			continue
		}
		if !requestServiceStop(serviceID) && serviceManifest[serviceID].class == classCritical {
			return false
		}
	}
	return true
}

func requestServiceStop(serviceID uintptr) bool {
	stopCmd := serviceManifest[serviceID].stopCmd
	if stopCmd == 0 {
		return true
	}

	ep := sysSvcLookup(&serviceManifest[serviceID].name[0], uintptr(len(serviceManifest[serviceID].name)))
	if ep == sysErr {
		return false
	}

	req := [1]byte{stopCmd}
	if sysIpcSend(ep, &req[0], 1) == sysErr {
		return false
	}

	logServiceAction(msgSvcMgrStop[:], serviceID)
	setServiceState(serviceID, stateStopping)
	return true
}

func setServiceState(serviceID uintptr, state byte) {
	prev := serviceStates[serviceID]
	if prev == state {
		return
	}

	serviceStates[serviceID] = state
	serviceLastTick[serviceID] = runtimeTick()

	switch state {
	case stateStarting:
		serviceStarts[serviceID]++
	case stateFailed:
		serviceFailures[serviceID]++
	}

	logServiceState(serviceID, state)
}

func logServiceState(serviceID uintptr, state byte) {
	log(msgServicePrefix[:])
	log(serviceManifest[serviceID].name)
	log(msgSpace[:])
	log(stateLabel(state))
	log(msgNewline[:])
}

func logServiceAction(prefix []byte, serviceID uintptr) {
	log(prefix)
	log(serviceManifest[serviceID].name)
	log(msgNewline[:])
}

func stateLabel(state byte) []byte {
	switch state {
	case stateDeclared:
		return msgStateDeclared[:]
	case stateBlocked:
		return msgStateBlocked[:]
	case stateStarting:
		return msgStateStarting[:]
	case stateRunning:
		return msgStateRunning[:]
	case stateStopping:
		return msgStateStopping[:]
	case stateStopped:
		return msgStateStopped[:]
	default:
		return msgStateFailed[:]
	}
}

func runtimeTick() uintptr {
	tick := sysTimeNow()
	if tick == sysErr {
		return 0
	}
	return tick
}

func logUint(value uintptr) {
	var buf [20]byte
	i := len(buf)

	if value == 0 {
		i--
		buf[i] = '0'
	} else {
		for value != 0 {
			i--
			buf[i] = byte('0' + value%10)
			value /= 10
		}
	}

	log(buf[i:])
}

func logServiceSnapshot(serviceID uintptr) {
	log(msgProcPrefix[:])
	log(serviceManifest[serviceID].name)
	log(msgSpace[:])
	log(msgMetricStarts[:])
	logUint(uintptr(serviceStarts[serviceID]))
	log(msgSpace[:])
	log(msgMetricRest[:])
	logUint(uintptr(serviceRestarts[serviceID]))
	log(msgSpace[:])
	log(msgMetricFail[:])
	logUint(uintptr(serviceFailures[serviceID]))
	log(msgSpace[:])
	log(msgMetricReaps[:])
	logUint(uintptr(serviceReaps[serviceID]))
	log(msgSpace[:])
	log(msgMetricTick[:])
	logUint(serviceLastTick[serviceID])
	log(msgNewline[:])
}

//export goSpawnedThreadMain
func goSpawnedThreadMain() {
	switch spawnServiceID {
	case serviceTime:
		timeServiceMain()
	case serviceDiag:
		diagServiceMain()
	case serviceShell:
		shellMain()
	default:
		fail(msgGoInitErr[:])
	}
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
