package main

// tcpcheck <port>: connect over real wire TCP to the slirp host gateway
// (10.0.2.2) on the given port, exchange a payload with the host-side
// listener, and report the round trip.

var (
	msgNettOK      = []byte("NETT: tcp ok\n")
	msgNettErr     = []byte("NETT: tcp err\n")
	nettPayload    = []byte("rugo-tcp-hello")
	nettExpect     = []byte("tcp-hello-back")
	nettGatewayIP4 = [4]byte{10, 0, 2, 2}
)

func parsePort(args string) (uint64, bool) {
	var port uint64
	if len(args) == 0 || len(args) > 5 {
		return 0, false
	}
	var i int
	for i = 0; i < len(args); i++ {
		c := args[i]
		if c < '0' || c > '9' {
			return 0, false
		}
		port = port*10 + uint64(c-'0')
	}
	if port == 0 || port > 65535 {
		return 0, false
	}
	return port, true
}

func tcpCheck(args string) bool {
	port, ok := parsePort(args)
	if !ok {
		log(msgNettErr)
		return false
	}

	sock := sysSocketOpen(netFamilyInet, socketStream)
	if sock == sysErr {
		log(msgNettErr)
		return false
	}

	var addr socketAddr
	addr.Family = netFamilyInet
	addr.Port = port
	addr.Addr[0] = nettGatewayIP4[0]
	addr.Addr[1] = nettGatewayIP4[1]
	addr.Addr[2] = nettGatewayIP4[2]
	addr.Addr[3] = nettGatewayIP4[3]

	if sysSocketConnect(sock, &addr) == sysErr {
		sysSocketClose(sock)
		log(msgNettErr)
		return false
	}

	// The handshake completes from the PIT-tick RX pump; retry the send
	// until the connection is established.
	sent := false
	var tries uintptr
	for tries = 0; tries < 400; tries++ {
		if sysSocketSend(sock, &nettPayload[0], uintptr(len(nettPayload))) == uintptr(len(nettPayload)) {
			sent = true
			break
		}
		if sysYield() != 0 {
			break
		}
	}
	if !sent {
		sysSocketClose(sock)
		log(msgNettErr)
		return false
	}

	var reply [32]byte
	var got uintptr
	for tries = 0; tries < 400; tries++ {
		n := sysSocketRecv(sock, &reply[0], uintptr(len(reply)))
		if n != sysErr && n > 0 {
			got = n
			break
		}
		if sysYield() != 0 {
			break
		}
	}
	okReply := got >= uintptr(len(nettExpect))
	if okReply {
		var i uintptr
		for i = 0; i < uintptr(len(nettExpect)); i++ {
			if reply[i] != nettExpect[i] {
				okReply = false
				break
			}
		}
	}
	if sysSocketClose(sock) == sysErr {
		okReply = false
	}
	if !okReply {
		log(msgNettErr)
		return false
	}
	log(msgNettOK)
	return true
}
