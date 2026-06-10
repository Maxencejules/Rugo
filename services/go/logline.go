package main

// lineBuilder composes one log line in a stack buffer so the whole line
// reaches the console in a single sysDebugWrite. Under preemptive
// scheduling, a line assembled from several writes can be spliced by
// other tasks' output mid-line.
type lineBuilder struct {
	n   uintptr
	buf [224]byte
}

func (lb *lineBuilder) add(part []byte) {
	var idx uintptr
	for idx = 0; idx < uintptr(len(part)); idx++ {
		if lb.n >= uintptr(len(lb.buf)) {
			return
		}
		lb.buf[lb.n] = part[idx]
		lb.n++
	}
}

func (lb *lineBuilder) addUint(value uintptr) {
	var tmp [20]byte
	i := uintptr(len(tmp))
	if value == 0 {
		i--
		tmp[i] = '0'
	} else {
		for value != 0 {
			i--
			tmp[i] = byte('0' + value%10)
			value /= 10
		}
	}
	lb.add(tmp[i:])
}

func (lb *lineBuilder) emit() {
	if lb.n == 0 {
		return
	}
	log(lb.buf[:lb.n])
}
