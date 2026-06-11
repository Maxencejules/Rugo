package main

// Coreutils dispatch: ls/cat/echo/ps/wc run as REAL external programs
// from the package store on disk (gap item 8). The shell spawns the
// on-disk ELF with the command's argument string and reaps it; all
// visible output comes from the program itself. A single `left | right`
// pipeline runs the two programs sequentially through a kernel pipe
// (the 512-byte ring decouples them; true concurrency needs per-process
// address spaces).

const noFd = ^uintptr(0)

var (
	appNameEcho    = []byte("echo")
	appNameCat     = []byte("cat")
	appNameLs      = []byte("ls")
	appNamePs      = []byte("ps")
	appNameWc      = []byte("wc")
	msgShellRunErr = []byte("APP: run err\n")
	msgShellPipeOK = []byte("GOSH: pipe ok\n")
	defaultLsPath  = "/data"
)

func spawnRunIO(name []byte, args string, stdinFd uintptr, stdoutFd uintptr) bool {
	var argBuf [96]byte
	n := len(args)
	if n > len(argBuf) {
		n = len(argBuf)
	}
	var i int
	for i = 0; i < n; i++ {
		argBuf[i] = args[i]
	}
	var argPtr *byte
	if n > 0 {
		argPtr = &argBuf[0]
	}
	tid := sysSpawn(&name[0], uintptr(len(name)), argPtr, uintptr(n), stdinFd, stdoutFd)
	if tid == sysErr {
		log(msgShellRunErr)
		return false
	}
	if sysWait(tid, nil, 0) == sysErr {
		log(msgShellRunErr)
		return false
	}
	return true
}

func spawnRun(name []byte, args string) bool {
	return spawnRunIO(name, args, noFd, noFd)
}

func appByName(name string) ([]byte, bool) {
	switch name {
	case "echo":
		return appNameEcho, true
	case "cat":
		return appNameCat, true
	case "ls":
		return appNameLs, true
	case "ps":
		return appNamePs, true
	case "wc":
		return appNameWc, true
	}
	return nil, false
}

func splitFirstWord(s string) (string, string) {
	var i int
	for i = 0; i < len(s); i++ {
		if s[i] == ' ' {
			return s[:i], s[i+1:]
		}
	}
	return s, ""
}

// runPipeline executes `left | right`: left's output feeds right's stdin
// through a kernel pipe. Sequential in v1 - left runs to completion
// first, bounded by the pipe's 512-byte ring.
func runPipeline(left string, right string) bool {
	leftName, leftArgs := splitFirstWord(left)
	rightName, rightArgs := splitFirstWord(right)
	leftApp, ok1 := appByName(leftName)
	rightApp, ok2 := appByName(rightName)
	if !ok1 || !ok2 {
		log(msgShellRunErr)
		return false
	}

	pair := sysFsCtl(fsCtlPipe, nil, 0)
	if pair == sysErr {
		log(msgShellRunErr)
		return false
	}
	rfd := pair >> 8
	wfd := pair & 0xFF

	if !spawnRunIO(leftApp, leftArgs, noFd, wfd) {
		sysClose(rfd)
		return false
	}
	if !spawnRunIO(rightApp, rightArgs, rfd, noFd) {
		return false
	}
	log(msgShellPipeOK)
	return true
}
