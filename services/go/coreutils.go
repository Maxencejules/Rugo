package main

// Coreutils dispatch: ls/cat/echo/ps run as REAL external programs from
// the package store on disk (gap item 8). The shell spawns the on-disk
// ELF with the command's argument string and reaps it; all visible
// output comes from the program itself.

var (
	appNameEcho   = []byte("echo")
	appNameCat    = []byte("cat")
	appNameLs     = []byte("ls")
	appNamePs     = []byte("ps")
	msgShellRunErr = []byte("APP: run err\n")
	defaultLsPath  = "/data"
)

func spawnRun(name []byte, args string) bool {
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
	tid := sysSpawn(&name[0], uintptr(len(name)), argPtr, uintptr(n))
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
