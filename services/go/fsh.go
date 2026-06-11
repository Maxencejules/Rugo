package main

// Shell builtins for the writable /data file tree (SimpleFS v2).

var (
	msgFshWriteOK = []byte("FSH: write ok\n")
	msgFshCatOK   = []byte("FSH: cat ok\n")
	msgFshLsOK    = []byte("FSH: ls ok\n")
	msgFshMkOK    = []byte("FSH: mkdir ok\n")
	msgFshRmOK    = []byte("FSH: rm ok\n")
	msgFshChmodOK = []byte("FSH: chmod ok\n")
	msgFshErr     = []byte("FSH: err\n")
	msgFshDirMark = []byte("/")
)

func fshPath(arg string, buf []byte) bool {
	n := len(arg)
	if n == 0 || n+1 > len(buf) {
		return false
	}
	var i int
	for i = 0; i < n; i++ {
		buf[i] = arg[i]
	}
	buf[n] = 0
	return true
}

func fshWrite(args string) bool {
	sp := -1
	var i int
	for i = 0; i < len(args); i++ {
		if args[i] == ' ' {
			sp = i
			break
		}
	}
	if sp <= 0 || sp+1 >= len(args) {
		log(msgFshErr)
		return false
	}
	var path [96]byte
	if !fshPath(args[:sp], path[:]) {
		log(msgFshErr)
		return false
	}
	fd := sysOpen(&path[0], openWriteOnly|openCreate, 0)
	if fd == sysErr {
		log(msgFshErr)
		return false
	}
	text := args[sp+1:]
	var data [96]byte
	for i = 0; i < len(text) && i < len(data); i++ {
		data[i] = text[i]
	}
	ok := sysWrite(fd, &data[0], uintptr(i)) == uintptr(i)
	if sysClose(fd) == sysErr {
		ok = false
	}
	if !ok {
		log(msgFshErr)
		return false
	}
	log(msgFshWriteOK)
	return true
}

func fshCat(args string) bool {
	var path [96]byte
	if !fshPath(args, path[:]) {
		log(msgFshErr)
		return false
	}
	fd := sysOpen(&path[0], openReadOnly, 0)
	if fd == sysErr {
		log(msgFshErr)
		return false
	}
	var buf [192]byte
	n := sysRead(fd, &buf[0], uintptr(len(buf)))
	closed := sysClose(fd) != sysErr
	if n == sysErr || !closed {
		log(msgFshErr)
		return false
	}
	var lb lineBuilder
	lb.add(buf[:n])
	lb.add(msgNewline[:])
	lb.emit()
	log(msgFshCatOK)
	return true
}

func fshLs(args string) bool {
	var path [96]byte
	if !fshPath(args, path[:]) {
		log(msgFshErr)
		return false
	}
	fd := sysOpen(&path[0], openReadOnly, 0)
	if fd == sysErr {
		log(msgFshErr)
		return false
	}
	ok := true
	for {
		var recs [128]byte
		n := sysRead(fd, &recs[0], uintptr(len(recs)))
		if n == sysErr {
			ok = false
			break
		}
		if n == 0 {
			break
		}
		var off uintptr
		for off+32 <= n {
			var lb lineBuilder
			nameEnd := off
			for nameEnd < off+24 && recs[nameEnd] != 0 {
				nameEnd++
			}
			lb.add(recs[off:nameEnd])
			if recs[off+24] == 2 {
				lb.add(msgFshDirMark)
			}
			lb.add(msgNewline[:])
			lb.emit()
			off += 32
		}
	}
	if sysClose(fd) == sysErr || !ok {
		log(msgFshErr)
		return false
	}
	log(msgFshLsOK)
	return true
}

func fshCtl(op uintptr, args string, okMsg []byte) bool {
	var path [96]byte
	if !fshPath(args, path[:]) {
		log(msgFshErr)
		return false
	}
	if sysFsCtl(op, &path[0], 0) == sysErr {
		log(msgFshErr)
		return false
	}
	log(okMsg)
	return true
}

// fshChmod handles "fschmod <path> <mode 0..15>".
func fshChmod(args string) bool {
	sp := -1
	var i int
	for i = 0; i < len(args); i++ {
		if args[i] == ' ' {
			sp = i
		}
	}
	if sp <= 0 || sp+1 >= len(args) {
		log(msgFshErr)
		return false
	}
	mode := 0
	for i = sp + 1; i < len(args); i++ {
		if args[i] < '0' || args[i] > '9' {
			log(msgFshErr)
			return false
		}
		mode = mode*10 + int(args[i]-'0')
	}
	if mode > 15 {
		log(msgFshErr)
		return false
	}
	var path [96]byte
	if !fshPath(args[:sp], path[:]) {
		log(msgFshErr)
		return false
	}
	if sysFsCtl(fsCtlChmod, &path[0], uintptr(mode)) == sysErr {
		log(msgFshErr)
		return false
	}
	log(msgFshChmodOK)
	return true
}
