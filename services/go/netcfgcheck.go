package main

// Shell commands proving the DHCP and DNS clients (gap item 6
// remainder): the kernel sends real UDP over the VirtIO NIC and the
// shell polls for the parsed result.

var (
	msgNetdDhcpOK  = []byte("NETD: dhcp ok\n")
	msgNetdDhcpErr = []byte("NETD: dhcp err\n")
	msgNetdDnsOK   = []byte("NETD: dns ok\n")
	msgNetdDnsErr  = []byte("NETD: dns err\n")
)

const (
	netQueryDhcp = 1
	netQueryDns  = 2
	netQueryPoll = 3
)

func netQueryWait() bool {
	var i int
	for i = 0; i < 800; i++ {
		r := sysNetQuery(netQueryPoll, nil, 0)
		if r != sysErr {
			return true
		}
		sysYield()
	}
	return false
}

func dhcpCheck() bool {
	if sysNetQuery(netQueryDhcp, nil, 0) == sysErr {
		log(msgNetdDhcpErr)
		return false
	}
	if !netQueryWait() {
		log(msgNetdDhcpErr)
		return false
	}
	log(msgNetdDhcpOK)
	return true
}

// dnsCheck handles "dnscheck <name> <port>": an A query for name sent
// to the gateway at the given port (53 = the slirp resolver).
func dnsCheck(args string) bool {
	sp := -1
	var i int
	for i = 0; i < len(args); i++ {
		if args[i] == ' ' {
			sp = i
		}
	}
	if sp <= 0 || sp+1 >= len(args) {
		log(msgNetdDnsErr)
		return false
	}
	port := 0
	for i = sp + 1; i < len(args); i++ {
		if args[i] < '0' || args[i] > '9' {
			log(msgNetdDnsErr)
			return false
		}
		port = port*10 + int(args[i]-'0')
	}
	name := args[:sp]
	if len(name) == 0 || len(name) > 63 || port == 0 || port > 65535 {
		log(msgNetdDnsErr)
		return false
	}
	var buf [64]byte
	for i = 0; i < len(name); i++ {
		buf[i] = name[i]
	}
	arg3 := uintptr(len(name)) | uintptr(port)<<16
	if sysNetQuery(netQueryDns, &buf[0], arg3) == sysErr {
		log(msgNetdDnsErr)
		return false
	}
	if !netQueryWait() {
		log(msgNetdDnsErr)
		return false
	}
	log(msgNetdDnsOK)
	return true
}
