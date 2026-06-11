package main

// Spawn-stress probe: before any service starts, init spawns 8 throwaway
// workers (9 concurrent tasks with init itself - past the historical
// 6-slot static limit), then reaps them all. The workers' slots are
// reused by the services afterwards, keeping their historical tids.

const (
	serviceWorker     = uintptr(0xF7)
	spawnStressCount  = 8
	workerYieldRounds = 2
)

var (
	workerGoFlag uintptr

	msgSpawnStressOK  = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 's', 'p', 'a', 'w', 'n', ' ', 's', 't', 'r', 'e', 's', 's', ' ', 'o', 'k', ' ', 'n', '=', '8', '\n'}
	msgSpawnStressErr = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 's', 'p', 'a', 'w', 'n', ' ', 's', 't', 'r', 'e', 's', 's', ' ', 'e', 'r', 'r', '\n'}
)

func workerMain() {
	var round uintptr
	for round = 0; round < workerYieldRounds; round++ {
		if sysYield() != 0 {
			return
		}
	}
}

func runSpawnStress() {
	var spawned uintptr
	var idx uintptr
	for idx = 0; idx < spawnStressCount; idx++ {
		spawnAck = 0
		workerGoFlag = 0
		spawnServiceID = serviceWorker
		tid := sysThreadSpawn(spawnEntry)
		if tid == sysErr {
			fail(msgSpawnStressErr[:])
		}
		ackBudget := uintptr(64)
		for spawnAck == 0 && ackBudget > 0 {
			if sysYield() != 0 {
				fail(msgSpawnStressErr[:])
			}
			ackBudget--
		}
		if spawnAck == 0 {
			fail(msgSpawnStressErr[:])
		}
		workerGoFlag = 1
		spawned++
	}

	var reaped uintptr
	for reaped < spawned {
		if sysWait(waitAny, nil, 0) == sysErr {
			fail(msgSpawnStressErr[:])
		}
		reaped++
	}
	log(msgSpawnStressOK[:])
}
