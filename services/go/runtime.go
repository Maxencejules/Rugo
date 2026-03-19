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
	servicePkg
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
	phaseSession
	phaseShutdown
)

const phaseCount = 4

const (
	stateDeclared = iota
	stateBlocked
	stateStarting
	stateRunning
	stateReady
	stateFailed
	stateStopping
	stateStopped
)

const stateUnset = 0xFF
const taskUnset = 0xFF

const (
	requiredOptional = iota
	requiredBoot
)

const (
	initResultBootFailed = iota
	initResultSessionFailed
	initResultShutdownFailed
	initResultShutdownClean
)

const (
	serviceResultUnset = iota
	serviceResultOnline
	serviceResultRuntimeFailed
	serviceResultSpawnFailed
	serviceResultWedge
	serviceResultRestarting
	serviceResultOrderedStop
	serviceResultSessionDone
	serviceResultShutdownError
	serviceResultRestartExhausted
)

type serviceSpec struct {
	name         []byte
	role         []byte
	class        byte
	policy       byte
	required     byte
	deps         byte
	phase        byte
	startBudget  byte
	restartLimit byte
	stopCmd      byte
}

var (
	spawnEntry      uintptr
	spawnServiceID  uintptr
	bootFailed      uintptr
	bootOperational uintptr
	shutdownStarted uintptr
	shellComplete   uintptr

	serviceStates = [serviceCount]byte{stateUnset, stateUnset, stateUnset, stateUnset}
	serviceTasks  = [serviceCount]byte{taskUnset, taskUnset, taskUnset, taskUnset}

	serviceResults  [serviceCount]byte
	serviceRestarts [serviceCount]byte
	serviceStarts   [serviceCount]byte
	serviceFailures [serviceCount]byte
	serviceReaps    [serviceCount]byte
	serviceLastTick [serviceCount]uintptr
	shellRecycles   byte
	c5CleanupLogged byte

	managerCleanupInfo taskInfo
)

var (
	nameTimeSvc = [...]byte{'t', 'i', 'm', 'e', 's', 'v', 'c'}
	nameDiagSvc = [...]byte{'d', 'i', 'a', 'g', 's', 'v', 'c'}
	nameShell   = [...]byte{'s', 'h', 'e', 'l', 'l'}
	nameHijack  = [...]byte{'h', 'i', 'j', 'a', 'c', 'k'}
	roleTime    = [...]byte{'t', 'i', 'm', 'e'}
	roleDiag    = [...]byte{'d', 'i', 'a', 'g'}
	roleShell   = [...]byte{'s', 'h', 'e', 'l', 'l'}
	rolePkg     = [...]byte{'p', 'k', 'g'}
	replyOK     = [...]byte{'O', 'K'}
)

var serviceManifest = [...]serviceSpec{
	{
		name:         nameTimeSvc[:],
		role:         roleTime[:],
		class:        classCritical,
		policy:       restartOnFailure,
		required:     requiredBoot,
		deps:         0,
		phase:        phaseCore,
		startBudget:  8,
		restartLimit: 3,
		stopCmd:      cmdStop,
	},
	{
		name:         nameDiagSvc[:],
		role:         roleDiag[:],
		class:        classBestEffort,
		policy:       restartOnFailure,
		required:     requiredBoot,
		deps:         1 << serviceTime,
		phase:        phaseServices,
		startBudget:  8,
		restartLimit: 3,
		stopCmd:      cmdStop,
	},
	{
		name:         nameShell[:],
		role:         roleShell[:],
		class:        classBestEffort,
		policy:       restartOnFailure,
		required:     requiredBoot,
		deps:         (1 << serviceTime) | (1 << serviceDiag),
		phase:        phaseSession,
		startBudget:  12,
		restartLimit: 2,
		stopCmd:      0,
	},
	{
		name:         namePkgSvc[:],
		role:         rolePkg[:],
		class:        classBestEffort,
		policy:       restartOnFailure,
		required:     requiredOptional,
		deps:         1 << serviceTime,
		phase:        phaseServices,
		startBudget:  8,
		restartLimit: 3,
		stopCmd:      cmdStop,
	},
}

var restartBackoffYields = [...]uintptr{1, 2, 4}

var (
	msgGoInitStart       = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 's', 't', 'a', 'r', 't', '\n'}
	msgGoInitBootstrap   = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 'b', 'o', 'o', 't', 's', 't', 'r', 'a', 'p', '\n'}
	msgGoInitSpawn       = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 's', 'v', 'c', 'm', 'g', 'r', ' ', 'u', 'p', '\n'}
	msgGoInitOperational = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 'o', 'p', 'e', 'r', 'a', 't', 'i', 'o', 'n', 'a', 'l', '\n'}
	msgGoInitResult      = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 'r', 'e', 's', 'u', 'l', 't', ' '}
	msgGoInitReady       = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 'r', 'e', 'a', 'd', 'y', '\n'}
	msgGoInitErr         = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 'e', 'r', 'r', '\n'}

	msgSvcMgrStart = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 's', 't', 'a', 'r', 't', '\n'}
	msgSvcMgrShell = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 's', 'h', 'e', 'l', 'l', '\n'}
	msgSvcMgrReap  = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 'r', 'e', 'a', 'p', ' '}
	msgSvcMgrRetry = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 'r', 'e', 's', 't', 'a', 'r', 't', ' '}
	msgSvcMgrStop  = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 's', 't', 'o', 'p', ' '}
	msgSvcMgrWedge = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 'w', 'e', 'd', 'g', 'e', ' '}
	msgSvcMgrClass = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 'c', 'l', 'a', 's', 's', ' '}
	msgSvcMgrPlan  = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 'p', 'l', 'a', 'n', ' '}
	msgSvcMgrPhase = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 'p', 'h', 'a', 's', 'e', ' '}
	msgSvcMgrErr   = [...]byte{'G', 'O', 'S', 'V', 'C', 'M', ':', ' ', 'e', 'r', 'r', '\n'}

	msgServicePrefix = [...]byte{'S', 'V', 'C', ':', ' '}
	msgProcPrefix    = [...]byte{'P', 'R', 'O', 'C', ':', ' '}
	msgTaskPrefix    = [...]byte{'T', 'A', 'S', 'K', ':', ' '}
	msgMetricStarts  = [...]byte{'s', '='}
	msgMetricRest    = [...]byte{'r', '='}
	msgMetricFail    = [...]byte{'f', '='}
	msgMetricReaps   = [...]byte{'x', '='}
	msgMetricTick    = [...]byte{'t', '='}
	msgMetricTid     = [...]byte{'t', 'i', 'd', '='}
	msgMetricParent  = [...]byte{'p', 'a', 'r', 'e', 'n', 't', '='}
	msgMetricClass   = [...]byte{'c', 'l', 's', '='}
	msgMetricState   = [...]byte{'s', 't', '='}
	msgMetricRun     = [...]byte{'r', 'u', 'n', '='}
	msgMetricYield   = [...]byte{'y', '='}
	msgMetricBlock   = [...]byte{'b', 'l', 'k', '='}
	msgMetricSend    = [...]byte{'t', 'x', '='}
	msgMetricRecv    = [...]byte{'r', 'x', '='}
	msgMetricEp      = [...]byte{'e', 'p', '='}
	msgMetricDomain  = [...]byte{'d', 'o', 'm', '='}
	msgMetricCap     = [...]byte{'c', 'a', 'p', '='}
	msgMetricFd      = [...]byte{'f', 'd', '='}
	msgMetricSock    = [...]byte{'s', 'o', 'c', 'k', '='}
	msgMetricSvc     = [...]byte{'s', 'v', 'c', '='}
	msgMetricRole    = [...]byte{'r', 'o', 'l', 'e', '='}
	msgMetricPhase   = [...]byte{'p', 'h', 'a', 's', 'e', '='}
	msgMetricNeed    = [...]byte{'n', 'e', 'e', 'd', '='}
	msgMetricDeps    = [...]byte{'d', 'e', 'p', 's', '='}
	msgMetricPolicy  = [...]byte{'r', 's', 't', '='}
	msgMetricResult  = [...]byte{'r', 'e', 's', '='}
	msgSpace         = [...]byte{' '}
	msgComma         = [...]byte{','}
	msgSlash         = [...]byte{'/'}
	msgNewline       = [...]byte{'\n'}

	msgStateDeclared    = [...]byte{'d', 'e', 'c', 'l', 'a', 'r', 'e', 'd'}
	msgStateBlocked     = [...]byte{'b', 'l', 'o', 'c', 'k', 'e', 'd'}
	msgStateStarting    = [...]byte{'s', 't', 'a', 'r', 't', 'i', 'n', 'g'}
	msgStateRunning     = [...]byte{'r', 'u', 'n', 'n', 'i', 'n', 'g'}
	msgStateReady       = [...]byte{'r', 'e', 'a', 'd', 'y'}
	msgStateFailed      = [...]byte{'f', 'a', 'i', 'l', 'e', 'd'}
	msgStateStopping    = [...]byte{'s', 't', 'o', 'p', 'p', 'i', 'n', 'g'}
	msgStateStopped     = [...]byte{'s', 't', 'o', 'p', 'p', 'e', 'd'}
	msgTaskStateReady   = [...]byte{'r', 'e', 'a', 'd', 'y'}
	msgTaskStateBlocked = [...]byte{'b', 'l', 'o', 'c', 'k', 'e', 'd'}
	msgTaskStateExited  = [...]byte{'e', 'x', 'i', 't', 'e', 'd'}
	msgTaskStateDead    = [...]byte{'d', 'e', 'a', 'd'}
	msgClassCritical    = [...]byte{'c', 'r', 'i', 't', 'i', 'c', 'a', 'l'}
	msgClassBestEffort  = [...]byte{'b', 'e', 's', 't', '-', 'e', 'f', 'f', 'o', 'r', 't'}
	msgNeedRequired     = [...]byte{'r', 'e', 'q', 'u', 'i', 'r', 'e', 'd'}
	msgNeedOptional     = [...]byte{'o', 'p', 't', 'i', 'o', 'n', 'a', 'l'}
	msgPolicyNever      = [...]byte{'n', 'e', 'v', 'e', 'r'}
	msgPolicyFail       = [...]byte{'o', 'n', '-', 'f', 'a', 'i', 'l', 'u', 'r', 'e'}
	msgPolicyAlways     = [...]byte{'a', 'l', 'w', 'a', 'y', 's'}
	msgPhaseCore        = [...]byte{'c', 'o', 'r', 'e'}
	msgPhaseServices    = [...]byte{'b', 'a', 's', 'e'}
	msgPhaseSession     = [...]byte{'s', 'e', 's', 's', 'i', 'o', 'n'}
	msgPhaseShutdown    = [...]byte{'s', 'h', 'u', 't', 'd', 'o', 'w', 'n'}
	msgDepsNone         = [...]byte{'n', 'o', 'n', 'e'}
	msgInitBootFailed   = [...]byte{'b', 'o', 'o', 't', '-', 'f', 'a', 'i', 'l', 'e', 'd'}
	msgInitSessFailed   = [...]byte{'s', 'e', 's', 's', 'i', 'o', 'n', '-', 'f', 'a', 'i', 'l', 'e', 'd'}
	msgInitShdnFailed   = [...]byte{'s', 'h', 'u', 't', 'd', 'o', 'w', 'n', '-', 'f', 'a', 'i', 'l', 'e', 'd'}
	msgInitShdnClean    = [...]byte{'s', 'h', 'u', 't', 'd', 'o', 'w', 'n', '-', 'c', 'l', 'e', 'a', 'n'}
	msgResultNone       = [...]byte{'n', 'o', 'n', 'e'}
	msgResultOnline     = [...]byte{'o', 'n', 'l', 'i', 'n', 'e'}
	msgResultRuntime    = [...]byte{'r', 'u', 'n', 't', 'i', 'm', 'e', '-', 'f', 'a', 'i', 'l', 'e', 'd'}
	msgResultSpawn      = [...]byte{'s', 'p', 'a', 'w', 'n', '-', 'f', 'a', 'i', 'l', 'e', 'd'}
	msgResultWedge      = [...]byte{'w', 'e', 'd', 'g', 'e'}
	msgResultRestarting = [...]byte{'r', 'e', 's', 't', 'a', 'r', 't', 'i', 'n', 'g'}
	msgResultStop       = [...]byte{'o', 'r', 'd', 'e', 'r', 'e', 'd', '-', 's', 't', 'o', 'p'}
	msgResultSession    = [...]byte{'s', 'e', 's', 's', 'i', 'o', 'n', '-', 'd', 'o', 'n', 'e'}
	msgResultShutdown   = [...]byte{'s', 'h', 'u', 't', 'd', 'o', 'w', 'n', '-', 'e', 'r', 'r', 'o', 'r'}
	msgResultExhausted  = [...]byte{'r', 'e', 's', 't', 'a', 'r', 't', '-', 'e', 'x', 'h', 'a', 'u', 's', 't', 'e', 'd'}
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
	result := serviceManagerMain(order)
	logInitResult(result)
	if result != initResultShutdownClean {
		fail(msgGoInitErr[:])
	}
	log(msgGoInitReady[:])
	sysThreadExit()
	fail(msgGoInitErr[:])
}

func serviceManagerMain(order [serviceCount]byte) byte {
	log(msgSvcMgrStart[:])

	var idx uintptr
	var phaseLogged [phaseCount]byte
	for idx = 0; idx < serviceCount; idx++ {
		serviceID := uintptr(order[idx])
		setServiceState(serviceID, stateDeclared)
		logServicePlan(serviceID)
	}

	var liveChildren uintptr
	for bootFailed == 0 {
		launched, failed := launchEligibleServices(order, bootOperational != 0, &phaseLogged)
		liveChildren += launched
		if failed {
			return managerError(managerFailureResult())
		}

		if bootOperational == 0 && allBootServicesReady() {
			bootOperational = 1
			log(msgGoInitOperational[:])
			continue
		}

		if shellComplete != 0 && shutdownStarted == 0 {
			shutdownStarted = 1
			logManagerPhase(phaseShutdown, &phaseLogged)
			if !beginShutdown(order) {
				return managerError(initResultShutdownFailed)
			}
		}

		if liveChildren == 0 {
			break
		}

		serviceID, restart, ok := reapService()
		if !ok {
			return managerError(managerFailureResult())
		}
		liveChildren--

		if restart {
			logServiceAction(msgSvcMgrRetry[:], serviceID)
			if !scheduleRestart(serviceID) {
				if serviceNeedsSuccess(serviceID) {
					return managerError(managerFailureResult())
				}
				continue
			}
			setServiceState(serviceID, stateDeclared)
			continue
		}
	}

	if bootFailed != 0 {
		return managerError(managerFailureResult())
	}

	if !serviceManagerCleanExit() {
		return managerError(managerFailureResult())
	}

	return initResultShutdownClean
}

func launchEligibleServices(order [serviceCount]byte, sessionAllowed bool, phaseLogged *[phaseCount]byte) (uintptr, bool) {
	var idx uintptr
	var liveChildren uintptr

	for idx = 0; idx < serviceCount; idx++ {
		serviceID := uintptr(order[idx])
		spec := serviceManifest[serviceID]
		if !phaseAllowed(spec.phase, sessionAllowed) {
			continue
		}
		if !serviceLaunchable(serviceID) {
			continue
		}
		if !depsReady(spec.deps) {
			setServiceState(serviceID, stateBlocked)
			continue
		}

		logManagerPhase(spec.phase, phaseLogged)
		if serviceID == serviceShell {
			log(msgSvcMgrShell[:])
		}

		if !launchService(serviceID) {
			if serviceNeedsSuccess(serviceID) {
				return liveChildren, true
			}
			continue
		}
		liveChildren++
	}

	return liveChildren, false
}

func phaseAllowed(phase byte, sessionAllowed bool) bool {
	if phase == phaseSession {
		return sessionAllowed
	}
	return true
}

func serviceLaunchable(serviceID uintptr) bool {
	if serviceTasks[serviceID] != taskUnset {
		return false
	}
	switch serviceStates[serviceID] {
	case stateDeclared, stateBlocked:
		return true
	default:
		return false
	}
}

func allBootServicesReady() bool {
	var serviceID uintptr
	for serviceID = 0; serviceID < serviceCount; serviceID++ {
		spec := serviceManifest[serviceID]
		if spec.required != requiredBoot || spec.phase == phaseSession {
			continue
		}
		if serviceStates[serviceID] != stateReady {
			return false
		}
	}
	return true
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

func depsReady(mask byte) bool {
	var serviceID uintptr
	for serviceID = 0; serviceID < serviceCount; serviceID++ {
		if mask&(1<<serviceID) == 0 {
			continue
		}
		if serviceStates[serviceID] != stateReady {
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
			setServiceResult(serviceID, serviceResultSpawnFailed)
			setServiceState(serviceID, stateFailed)
			serviceTasks[serviceID] = taskUnset
		} else {
			serviceTasks[serviceID] = byte(tid)
			if !applyServiceIsolation(serviceID, tid) {
				fail(msgSvcMgrErr[:])
			}
			if !applyServiceScheduling(serviceID, tid) {
				fail(msgSvcMgrErr[:])
			}
		}

		budget := uintptr(serviceManifest[serviceID].startBudget)
		for serviceStates[serviceID] != stateReady && serviceStates[serviceID] != stateFailed && bootFailed == 0 {
			if budget == 0 {
				logServiceAction(msgSvcMgrWedge[:], serviceID)
				setServiceResult(serviceID, serviceResultWedge)
				setServiceState(serviceID, stateFailed)
				break
			}
			if sysYield() != 0 {
				fail(msgSvcMgrErr[:])
			}
			budget--
		}

		switch serviceStates[serviceID] {
		case stateReady:
			return true
		case stateFailed:
			if serviceTasks[serviceID] != taskUnset {
				return true
			}
			if !scheduleRestart(serviceID) {
				if serviceResultForFailure(serviceResults[serviceID]) == serviceResultRuntimeFailed {
					setServiceResult(serviceID, serviceResultRestartExhausted)
				}
				return false
			}
		default:
			return false
		}
	}
}

func reapService() (uintptr, bool, bool) {
	tid := sysWait(waitAny, nil, 0)
	if tid == sysErr {
		return serviceCount, false, false
	}

	serviceID := serviceByTask(tid)
	if serviceID == serviceCount {
		return serviceCount, false, false
	}

	if !verifyReapedServiceCleanup(serviceID, tid) {
		return serviceCount, false, false
	}

	serviceTasks[serviceID] = taskUnset
	serviceReaps[serviceID]++

	switch serviceStates[serviceID] {
	case stateStopped:
		logServiceStateAction(msgSvcMgrReap[:], serviceID, stateStopped)
		return serviceID, false, true
	case stateFailed:
		logServiceStateAction(msgSvcMgrReap[:], serviceID, stateFailed)
		return serviceID, true, true
	default:
		setServiceState(serviceID, stateFailed)
		logServiceStateAction(msgSvcMgrReap[:], serviceID, stateFailed)
		return serviceID, true, true
	}
}

func verifyReapedServiceCleanup(serviceID uintptr, tid uintptr) bool {
	if serviceID != serviceShell || shellComplete == 0 {
		return true
	}
	if c5CleanupLogged != 0 {
		return true
	}

	if sysProcInfo(tid, &managerCleanupInfo) == sysErr {
		return false
	}
	if managerCleanupInfo.EndpointCount != 0 || managerCleanupInfo.FdCount != 0 || managerCleanupInfo.SocketCount != 0 {
		return false
	}

	c5CleanupLogged = 1
	log(msgIsoC5Cleanup[:])
	return true
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
	return serviceManifest[serviceID].required == requiredBoot
}

func scheduleRestart(serviceID uintptr) bool {
	policy := serviceManifest[serviceID].policy
	if policy == restartNever {
		return false
	}
	if serviceRestarts[serviceID] >= serviceManifest[serviceID].restartLimit {
		setServiceResult(serviceID, serviceResultRestartExhausted)
		return false
	}

	backoff := restartBackoffYields[serviceRestarts[serviceID]]
	serviceRestarts[serviceID]++
	if !yieldCount(backoff) {
		return false
	}
	setServiceResult(serviceID, serviceResultRestarting)
	return true
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
		if !serviceIsActive(serviceID) {
			continue
		}
		if !requestServiceStop(serviceID) && serviceNeedsSuccess(serviceID) {
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
		setServiceResult(serviceID, serviceResultShutdownError)
		return false
	}

	req := [1]byte{stopCmd}
	if sysIpcSend(ep, &req[0], 1) == sysErr {
		setServiceResult(serviceID, serviceResultShutdownError)
		return false
	}

	logServiceAction(msgSvcMgrStop[:], serviceID)
	setServiceResult(serviceID, serviceResultOrderedStop)
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
	case stateReady:
		setServiceResult(serviceID, serviceResultOnline)
	case stateFailed:
		serviceFailures[serviceID]++
		setServiceResult(serviceID, serviceResultForFailure(serviceResults[serviceID]))
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

func logServiceStateAction(prefix []byte, serviceID uintptr, state byte) {
	log(prefix)
	log(serviceManifest[serviceID].name)
	log(msgSpace[:])
	log(stateLabel(state))
	log(msgSpace[:])
	log(msgMetricResult[:])
	log(serviceResultLabel(serviceResults[serviceID]))
	log(msgNewline[:])
}

func applyServiceIsolation(serviceID uintptr, tid uintptr) bool {
	cfg := isolationConfig{}

	switch serviceID {
	case serviceTime:
		cfg.DomainID = 1
		cfg.CapabilityFlags = 0
		cfg.Limits = packIsolationLimits(0, 0, 1)
	case serviceDiag:
		cfg.DomainID = 2
		cfg.CapabilityFlags = 0
		cfg.Limits = packIsolationLimits(0, 0, 1)
	case serviceShell:
		cfg.DomainID = 3
		cfg.CapabilityFlags = taskCapStorage | taskCapNetwork
		cfg.Limits = packIsolationLimits(2, 3, 2)
	case servicePkg:
		cfg.DomainID = 4
		cfg.CapabilityFlags = taskCapStorage
		cfg.Limits = packIsolationLimits(3, 0, 1)
	default:
		return false
	}

	return sysIsolationConfig(tid, &cfg) != sysErr
}

func applyServiceScheduling(serviceID uintptr, tid uintptr) bool {
	class := schedClassForService(serviceID)
	if sysSchedSet(tid, class) == sysErr {
		return false
	}
	log(msgSvcMgrClass[:])
	log(serviceManifest[serviceID].name)
	log(msgSpace[:])
	log(schedClassLabel(class))
	log(msgNewline[:])
	return true
}

func logManagerPhase(phase byte, phaseLogged *[phaseCount]byte) {
	if phaseLogged[phase] != 0 {
		return
	}
	phaseLogged[phase] = 1
	log(msgSvcMgrPhase[:])
	log(phaseLabel(phase))
	log(msgNewline[:])
}

func logServicePlan(serviceID uintptr) {
	spec := serviceManifest[serviceID]

	log(msgSvcMgrPlan[:])
	log(spec.name)
	log(msgSpace[:])
	log(msgMetricRole[:])
	log(spec.role)
	log(msgSpace[:])
	log(msgMetricPhase[:])
	log(phaseLabel(spec.phase))
	log(msgSpace[:])
	log(msgMetricNeed[:])
	log(requiredLabel(spec.required))
	log(msgSpace[:])
	log(msgMetricDeps[:])
	logDeps(spec.deps)
	log(msgSpace[:])
	log(msgMetricPolicy[:])
	log(policyLabel(spec.policy))
	log(msgSlash[:])
	logUint(uintptr(spec.restartLimit))
	log(msgNewline[:])
}

func logDeps(mask byte) {
	if mask == 0 {
		log(msgDepsNone[:])
		return
	}

	first := true
	var serviceID uintptr
	for serviceID = 0; serviceID < serviceCount; serviceID++ {
		if mask&(1<<serviceID) == 0 {
			continue
		}
		if !first {
			log(msgComma[:])
		}
		log(serviceManifest[serviceID].name)
		first = false
	}
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
	case stateReady:
		return msgStateReady[:]
	case stateStopping:
		return msgStateStopping[:]
	case stateStopped:
		return msgStateStopped[:]
	default:
		return msgStateFailed[:]
	}
}

func schedClassForService(serviceID uintptr) uintptr {
	if serviceManifest[serviceID].class == classCritical {
		return schedClassCritical
	}
	return schedClassBestEffort
}

func schedClassLabel(class uintptr) []byte {
	if class == schedClassCritical {
		return msgClassCritical[:]
	}
	return msgClassBestEffort[:]
}

func requiredLabel(required byte) []byte {
	if required == requiredBoot {
		return msgNeedRequired[:]
	}
	return msgNeedOptional[:]
}

func policyLabel(policy byte) []byte {
	switch policy {
	case restartNever:
		return msgPolicyNever[:]
	case restartAlways:
		return msgPolicyAlways[:]
	default:
		return msgPolicyFail[:]
	}
}

func phaseLabel(phase byte) []byte {
	switch phase {
	case phaseCore:
		return msgPhaseCore[:]
	case phaseSession:
		return msgPhaseSession[:]
	case phaseShutdown:
		return msgPhaseShutdown[:]
	default:
		return msgPhaseServices[:]
	}
}

func serviceIsActive(serviceID uintptr) bool {
	state := serviceStates[serviceID]
	return state == stateRunning || state == stateReady
}

func serviceManagerCleanExit() bool {
	var serviceID uintptr
	for serviceID = 0; serviceID < serviceCount; serviceID++ {
		state := serviceStates[serviceID]
		if serviceNeedsSuccess(serviceID) {
			if state != stateStopped {
				return false
			}
			continue
		}
		if serviceTasks[serviceID] != taskUnset {
			return false
		}
		if state != stateStopped && state != stateFailed && state != stateDeclared && state != stateBlocked {
			return false
		}
	}
	return true
}

func managerFailureResult() byte {
	if shutdownStarted != 0 {
		return initResultShutdownFailed
	}
	if bootOperational != 0 {
		return initResultSessionFailed
	}
	return initResultBootFailed
}

func managerError(result byte) byte {
	log(msgSvcMgrErr[:])
	return result
}

func logInitResult(result byte) {
	log(msgGoInitResult[:])
	log(initResultLabel(result))
	log(msgNewline[:])
}

func initResultLabel(result byte) []byte {
	switch result {
	case initResultBootFailed:
		return msgInitBootFailed[:]
	case initResultSessionFailed:
		return msgInitSessFailed[:]
	case initResultShutdownFailed:
		return msgInitShdnFailed[:]
	default:
		return msgInitShdnClean[:]
	}
}

func setServiceResult(serviceID uintptr, result byte) {
	serviceResults[serviceID] = result
}

func serviceResultForFailure(current byte) byte {
	switch current {
	case serviceResultSpawnFailed, serviceResultWedge, serviceResultShutdownError, serviceResultRestartExhausted:
		return current
	default:
		return serviceResultRuntimeFailed
	}
}

func serviceResultLabel(result byte) []byte {
	switch result {
	case serviceResultOnline:
		return msgResultOnline[:]
	case serviceResultRuntimeFailed:
		return msgResultRuntime[:]
	case serviceResultSpawnFailed:
		return msgResultSpawn[:]
	case serviceResultWedge:
		return msgResultWedge[:]
	case serviceResultRestarting:
		return msgResultRestarting[:]
	case serviceResultOrderedStop:
		return msgResultStop[:]
	case serviceResultSessionDone:
		return msgResultSession[:]
	case serviceResultShutdownError:
		return msgResultShutdown[:]
	case serviceResultRestartExhausted:
		return msgResultExhausted[:]
	default:
		return msgResultNone[:]
	}
}

func markServiceFailed(serviceID uintptr) {
	setServiceResult(serviceID, serviceResultRuntimeFailed)
	setServiceState(serviceID, stateFailed)
}

func taskStateLabel(state uint64) []byte {
	switch state {
	case taskStateReady:
		return msgTaskStateReady[:]
	case taskStateRunning:
		return msgStateRunning[:]
	case taskStateBlocked:
		return msgTaskStateBlocked[:]
	case taskStateExited:
		return msgTaskStateExited[:]
	default:
		return msgTaskStateDead[:]
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
	log(msgSpace[:])
	log(msgMetricSvc[:])
	log(stateLabel(serviceStates[serviceID]))
	log(msgSpace[:])
	log(msgMetricResult[:])
	log(serviceResultLabel(serviceResults[serviceID]))
	log(msgNewline[:])
}

func logKernelTaskSnapshot(serviceID uintptr) bool {
	tid := serviceTasks[serviceID]
	if tid == taskUnset {
		return false
	}

	var info taskInfo
	if sysProcInfo(uintptr(tid), &info) == sysErr {
		return false
	}

	log(msgTaskPrefix[:])
	log(serviceManifest[serviceID].name)
	log(msgSpace[:])
	log(msgMetricTid[:])
	logUint(uintptr(info.TID))
	log(msgSpace[:])
	log(msgMetricParent[:])
	logUint(uintptr(info.ParentTID))
	log(msgSpace[:])
	log(msgMetricClass[:])
	log(schedClassLabel(uintptr(info.SchedClass)))
	log(msgSpace[:])
	log(msgMetricState[:])
	log(taskStateLabel(info.State))
	log(msgSpace[:])
	log(msgMetricRun[:])
	logUint(uintptr(info.DispatchCount))
	log(msgSpace[:])
	log(msgMetricYield[:])
	logUint(uintptr(info.YieldCount))
	log(msgSpace[:])
	log(msgMetricBlock[:])
	logUint(uintptr(info.BlockCount))
	log(msgSpace[:])
	log(msgMetricSend[:])
	logUint(uintptr(info.IpcSendCount))
	log(msgSpace[:])
	log(msgMetricRecv[:])
	logUint(uintptr(info.IpcRecvCount))
	log(msgSpace[:])
	log(msgMetricEp[:])
	logUint(uintptr(info.EndpointCount))
	log(msgSpace[:])
	log(msgMetricDomain[:])
	logUint(uintptr(info.DomainID))
	log(msgSpace[:])
	log(msgMetricCap[:])
	logUint(uintptr(info.CapabilityFlags))
	log(msgSpace[:])
	log(msgMetricFd[:])
	logUint(uintptr(info.FdCount))
	log(msgSpace[:])
	log(msgMetricSock[:])
	logUint(uintptr(info.SocketCount))
	log(msgNewline[:])
	return true
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
	case servicePkg:
		pkgServiceMain()
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
