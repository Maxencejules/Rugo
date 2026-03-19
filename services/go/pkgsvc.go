package main

const cmdPkg = 'P'

const (
	pkgStateSize      = 28
	platformStateSize = 28
)

const (
	pkgInstallBaseShell = 1 << iota
	pkgInstallNetTools
	pkgInstallMediaSuite
)

const (
	platformCapSnapshot = 1 << iota
	platformCapResize
	platformCapXattr
	platformCapReflink
	platformCapNegotiation
)

const (
	pkgMagicP      = 'P'
	pkgMagicS      = 'S'
	pkgMagicT      = 'T'
	pkgMagic1      = '1'
	platformMagicP = 'P'
	platformMagicL = 'L'
	platformMagicT = 'T'
	platformMagic1 = '1'
)

const pkgMetaPayloadV1 = "stable|seq=1|key=1|catalog=3|pkgs=base-shell@1.0.0,net-tools@1.0.0"
const pkgRotatePayload = "rotate|seq=2|keys=1,2|catalog=4|promote=media-suite@2.0.0"
const pkgMetaPayloadV2 = "stable|seq=2|key=2|catalog=4|pkgs=base-shell@1.1.0,net-tools@1.0.0,media-suite@2.0.0"
const pkgXattrPayloadV1 = "channel=stable"
const pkgXattrPayloadV2 = "channel=stable-v2"

var (
	namePkgSvc = [...]byte{'p', 'k', 'g', 's', 'v', 'c'}

	pathPkgState = [...]byte{'/', 'r', 'u', 'n', 't', 'i', 'm', 'e', '/', 'p', 'k', 'g', 's', 't', 'a', 't', 'e', '.', 'b', 'i', 'n', 0}
	pathPlatform = [...]byte{'/', 'r', 'u', 'n', 't', 'i', 'm', 'e', '/', 'p', 'l', 'a', 't', 'f', 'o', 'r', 'm', '.', 'b', 'i', 'n', 0}

	keySlotOne = [...]byte{'r', 'u', 'g', 'o', '-', 'x', '3', '-', 'k', 'e', 'y', '-', '1'}
	keySlotTwo = [...]byte{'r', 'u', 'g', 'o', '-', 'x', '3', '-', 'k', 'e', 'y', '-', '2'}

	pkgMetaSigV1 = [32]byte{
		0x3a, 0x48, 0xbb, 0xc5, 0x35, 0x78, 0xa0, 0x3a,
		0x77, 0xc8, 0x03, 0x3c, 0x3e, 0x5f, 0xcc, 0x33,
		0x1c, 0x6d, 0xfe, 0x06, 0x32, 0xc6, 0xc3, 0x78,
		0x54, 0x0e, 0xc2, 0xfa, 0x1e, 0xac, 0xa8, 0x56,
	}
	pkgRotateSigOld = [32]byte{
		0x92, 0x36, 0x6d, 0xf0, 0x3e, 0x30, 0x2c, 0x81,
		0xeb, 0xc6, 0xf7, 0x31, 0x0c, 0x9a, 0xec, 0xc2,
		0xcd, 0x26, 0x13, 0x89, 0xc3, 0x5f, 0x23, 0x4c,
		0xed, 0x8f, 0x35, 0x83, 0x5d, 0x3c, 0x26, 0xcd,
	}
	pkgRotateSigNew = [32]byte{
		0x39, 0x14, 0x72, 0x99, 0x61, 0x9b, 0xad, 0xf5,
		0x58, 0x78, 0x28, 0x32, 0x4e, 0x8f, 0x2c, 0x95,
		0x57, 0xf4, 0xe9, 0xec, 0x8a, 0xa8, 0x7c, 0xe9,
		0x4b, 0xfd, 0xb8, 0xba, 0xa6, 0x35, 0x41, 0x0c,
	}
	pkgMetaSigV2 = [32]byte{
		0xfd, 0xe8, 0xe0, 0x65, 0x1e, 0x69, 0x7d, 0xe1,
		0x93, 0x79, 0xaf, 0xd4, 0x1e, 0xe7, 0x33, 0x9d,
		0x68, 0xfb, 0xb4, 0xe7, 0x1b, 0xb7, 0x1a, 0x3d,
		0xe6, 0x25, 0xba, 0x07, 0xbd, 0x0e, 0x7b, 0xdd,
	}

	msgPkgSvcStart = [...]byte{'P', 'K', 'G', 'S', 'V', 'C', ':', ' ', 's', 't', 'a', 'r', 't', '\n'}
	msgPkgSvcReady = [...]byte{'P', 'K', 'G', 'S', 'V', 'C', ':', ' ', 'r', 'e', 'a', 'd', 'y', '\n'}
	msgPkgSvcStop  = [...]byte{'P', 'K', 'G', 'S', 'V', 'C', ':', ' ', 's', 't', 'o', 'p', '\n'}
	msgPkgSvcErr   = [...]byte{'P', 'K', 'G', 'S', 'V', 'C', ':', ' ', 'e', 'r', 'r', '\n'}

	msgUpdMetadata = [...]byte{'U', 'P', 'D', '3', ':', ' ', 'm', 'e', 't', 'a', 'd', 'a', 't', 'a', ' ', 'o', 'k', '\n'}
	msgUpdRotate   = [...]byte{'U', 'P', 'D', '3', ':', ' ', 'r', 'o', 't', 'a', 't', 'e', ' ', 'o', 'k', '\n'}
	msgUpdApply    = [...]byte{'U', 'P', 'D', '3', ':', ' ', 'a', 'p', 'p', 'l', 'y', ' ', 'o', 'k', '\n'}
	msgUpdRollback = [...]byte{'U', 'P', 'D', '3', ':', ' ', 'r', 'o', 'l', 'l', 'b', 'a', 'c', 'k', ' ', 'o', 'k', '\n'}

	msgCatCatalog      = [...]byte{'C', 'A', 'T', '3', ':', ' ', 'c', 'a', 't', 'a', 'l', 'o', 'g', ' ', 'o', 'k', '\n'}
	msgCatInstallBase  = [...]byte{'C', 'A', 'T', '3', ':', ' ', 'i', 'n', 's', 't', 'a', 'l', 'l', ' ', 'b', 'a', 's', 'e', ' ', 'o', 'k', '\n'}
	msgCatInstallNet   = [...]byte{'C', 'A', 'T', '3', ':', ' ', 'i', 'n', 's', 't', 'a', 'l', 'l', ' ', 'n', 'e', 't', ' ', 'o', 'k', '\n'}
	msgCatInstallMedia = [...]byte{'C', 'A', 'T', '3', ':', ' ', 'i', 'n', 's', 't', 'a', 'l', 'l', ' ', 'm', 'e', 'd', 'i', 'a', ' ', 'o', 'k', '\n'}
	msgCatStage        = [...]byte{'C', 'A', 'T', '3', ':', ' ', 's', 't', 'a', 'g', 'e', ' ', 'o', 'k', '\n'}
	msgCatTelemetry    = [...]byte{'C', 'A', 'T', '3', ':', ' ', 't', 'e', 'l', 'e', 'm', 'e', 't', 'r', 'y', ' ', 'o', 'k', '\n'}

	msgStorX3Snapshot = [...]byte{'S', 'T', 'O', 'R', 'X', '3', ':', ' ', 's', 'n', 'a', 'p', 's', 'h', 'o', 't', ' ', 'o', 'k', '\n'}
	msgStorX3Resize   = [...]byte{'S', 'T', 'O', 'R', 'X', '3', ':', ' ', 'r', 'e', 's', 'i', 'z', 'e', ' ', 'o', 'k', '\n'}
	msgStorX3Xattr    = [...]byte{'S', 'T', 'O', 'R', 'X', '3', ':', ' ', 'x', 'a', 't', 't', 'r', ' ', 'o', 'k', '\n'}
	msgStorX3Reflink  = [...]byte{'S', 'T', 'O', 'R', 'X', '3', ':', ' ', 'r', 'e', 'f', 'l', 'i', 'n', 'k', ' ', 'o', 'k', '\n'}
	msgStorX3Cap      = [...]byte{'S', 'T', 'O', 'R', 'X', '3', ':', ' ', 'c', 'a', 'p', ' ', 'o', 'k', '\n'}

	msgShellPkg = [...]byte{'G', 'O', 'S', 'H', ':', ' ', 'p', 'k', 'g', ' ', 'o', 'k', '\n'}
)

type pkgState struct {
	Phase          byte
	TrustedKey     byte
	BuildSeq       byte
	StableSeq      byte
	RollbackSeq    byte
	CatalogCount   byte
	InstalledMask  byte
	TelemetryCount byte
	MetadataDigest [8]byte
	CatalogDigest  [8]byte
}

type platformState struct {
	SnapshotSeq    byte
	ResizeUnits    byte
	XattrLen       byte
	ReflinkCount   byte
	CapabilityMask byte
	InstalledMask  byte
	SnapshotDigest [8]byte
	XattrDigest    [8]byte
}

func requestPkg(serviceEP uintptr, replyEP uintptr) bool {
	return requestServiceCommand(serviceEP, replyEP, cmdPkg)
}

func pkgServiceMain() {
	log(msgPkgSvcStart[:])

	serviceEP := sysIpcEndpointCreate()
	if serviceEP == sysErr {
		markServiceFailed(servicePkg)
		fail(msgPkgSvcErr[:])
	}
	if sysSvcRegister(&namePkgSvc[0], uintptr(len(namePkgSvc)), serviceEP) == sysErr {
		markServiceFailed(servicePkg)
		fail(msgPkgSvcErr[:])
	}

	setServiceState(servicePkg, stateRunning)
	log(msgPkgSvcReady[:])
	setServiceState(servicePkg, stateReady)

	for bootFailed == 0 {
		var req [8]byte
		n := sysIpcRecv(serviceEP, &req[0], uintptr(len(req)))
		if n == sysErr {
			markServiceFailed(servicePkg)
			fail(msgPkgSvcErr[:])
		}
		if n == 1 && req[0] == cmdStop {
			log(msgPkgSvcStop[:])
			break
		}
		if n != 2 || req[0] != cmdPkg {
			markServiceFailed(servicePkg)
			fail(msgPkgSvcErr[:])
		}
		if !runPkgFlow() {
			markServiceFailed(servicePkg)
			fail(msgPkgSvcErr[:])
		}
		if sysIpcSend(uintptr(req[1]), &replyOK[0], uintptr(len(replyOK))) == sysErr {
			markServiceFailed(servicePkg)
			fail(msgPkgSvcErr[:])
		}
	}

	setServiceState(servicePkg, stateStopping)
	setServiceState(servicePkg, stateStopped)
	sysThreadExit()
	fail(msgPkgSvcErr[:])
}

func runPkgFlow() bool {
	var state pkgState
	var platform platformState
	hasState := loadPkgState(&state)
	_ = loadPlatformState(&platform)

	if !hasState {
		return runPkgBootstrap(&state, &platform)
	}
	if state.Phase == 1 {
		return runPkgUpgrade(&state, &platform)
	}
	return verifySteadyPkgState(&state, &platform)
}

func runPkgBootstrap(state *pkgState, platform *platformState) bool {
	if !verifySignedPayload(pkgMetaPayloadV1, pkgMetaSigV1, keySlotOne[:]) {
		return false
	}
	log(msgUpdMetadata[:])

	state.Phase = 1
	state.TrustedKey = 1
	state.BuildSeq = 1
	state.StableSeq = 1
	state.RollbackSeq = 1
	state.CatalogCount = 3
	state.InstalledMask = pkgInstallBaseShell | pkgInstallNetTools
	state.TelemetryCount = 2
	state.MetadataDigest = digestPrefix8(sha256DigestString(pkgMetaPayloadV1))
	state.CatalogDigest = digestPrefix8(sha256DigestString(pkgMetaPayloadV1))

	log(msgCatCatalog[:])
	log(msgCatInstallBase[:])
	log(msgCatInstallNet[:])
	log(msgCatTelemetry[:])

	platform.SnapshotSeq = 1
	platform.ResizeUnits = 64
	platform.XattrLen = byte(len(pkgXattrPayloadV1))
	platform.ReflinkCount = 0
	platform.CapabilityMask = platformCapSnapshot | platformCapXattr | platformCapNegotiation
	platform.InstalledMask = state.InstalledMask
	platform.SnapshotDigest = digestPrefix8(sha256DigestString(pkgMetaPayloadV1))
	platform.XattrDigest = digestPrefix8(sha256DigestString(pkgXattrPayloadV1))

	log(msgStorX3Snapshot[:])
	log(msgStorX3Xattr[:])
	log(msgStorX3Cap[:])

	if !storePkgState(state) || !storePlatformState(platform) {
		return false
	}
	return verifySteadyPkgState(state, platform)
}

func runPkgUpgrade(state *pkgState, platform *platformState) bool {
	if state.TrustedKey != 1 || state.BuildSeq != 1 {
		return false
	}
	if !verifySignedPayload(pkgRotatePayload, pkgRotateSigOld, keySlotOne[:]) {
		return false
	}
	if !verifySignedPayload(pkgRotatePayload, pkgRotateSigNew, keySlotTwo[:]) {
		return false
	}
	log(msgUpdRotate[:])
	if !verifySignedPayload(pkgMetaPayloadV2, pkgMetaSigV2, keySlotTwo[:]) {
		return false
	}
	log(msgUpdMetadata[:])

	log(msgCatCatalog[:])
	log(msgCatStage[:])
	log(msgCatInstallMedia[:])
	log(msgCatTelemetry[:])

	platform.SnapshotSeq = 2
	platform.ResizeUnits = 96
	platform.XattrLen = byte(len(pkgXattrPayloadV2))
	platform.ReflinkCount = 1
	platform.CapabilityMask = platformCapSnapshot | platformCapResize | platformCapXattr | platformCapReflink | platformCapNegotiation
	platform.InstalledMask = pkgInstallBaseShell | pkgInstallNetTools | pkgInstallMediaSuite
	platform.SnapshotDigest = digestPrefix8(sha256DigestString(pkgMetaPayloadV2))
	platform.XattrDigest = digestPrefix8(sha256DigestString(pkgXattrPayloadV2))

	log(msgStorX3Resize[:])
	log(msgStorX3Reflink[:])
	log(msgStorX3Xattr[:])
	log(msgStorX3Cap[:])
	log(msgUpdRollback[:])

	state.Phase = 2
	state.TrustedKey = 2
	state.BuildSeq = 2
	state.StableSeq = 2
	state.RollbackSeq = 1
	state.CatalogCount = 4
	state.InstalledMask = platform.InstalledMask
	state.TelemetryCount = 5
	state.MetadataDigest = digestPrefix8(sha256DigestString(pkgMetaPayloadV2))
	state.CatalogDigest = digestPrefix8(sha256DigestString(pkgMetaPayloadV2))

	if !storePlatformState(platform) || !storePkgState(state) {
		return false
	}
	if !verifySteadyPkgState(state, platform) {
		return false
	}
	log(msgUpdApply[:])
	return true
}

func verifySteadyPkgState(expectState *pkgState, expectPlatform *platformState) bool {
	var storedState pkgState
	var storedPlatform platformState
	if !loadPkgState(&storedState) || !loadPlatformState(&storedPlatform) {
		return false
	}
	if !pkgStateEqual(expectState, &storedState) {
		return false
	}
	if !platformStateEqual(expectPlatform, &storedPlatform) {
		return false
	}
	if storedState.TrustedKey == 1 {
		if !verifySignedPayload(pkgMetaPayloadV1, pkgMetaSigV1, keySlotOne[:]) {
			return false
		}
	} else if storedState.TrustedKey == 2 {
		if !verifySignedPayload(pkgMetaPayloadV2, pkgMetaSigV2, keySlotTwo[:]) {
			return false
		}
	} else {
		return false
	}
	if storedPlatform.CapabilityMask&platformCapSnapshot == 0 || storedPlatform.CapabilityMask&platformCapXattr == 0 {
		return false
	}
	return true
}

func verifySignedPayload(payload string, sig [32]byte, key []byte) bool {
	return digestEqual(keyedSignatureString(key, payload), sig)
}

func loadPkgState(out *pkgState) bool {
	var buf [pkgStateSize]byte
	n, ok := readRuntimeFile(&pathPkgState[0], buf[:])
	if !ok || n == 0 {
		return false
	}
	return decodePkgState(buf[:], n, out)
}

func storePkgState(state *pkgState) bool {
	var buf [pkgStateSize]byte
	encodePkgState(state, &buf)
	return writeRuntimeFile(&pathPkgState[0], buf[:])
}

func loadPlatformState(out *platformState) bool {
	var buf [platformStateSize]byte
	n, ok := readRuntimeFile(&pathPlatform[0], buf[:])
	if !ok || n == 0 {
		return false
	}
	return decodePlatformState(buf[:], n, out)
}

func storePlatformState(state *platformState) bool {
	var buf [platformStateSize]byte
	encodePlatformState(state, &buf)
	return writeRuntimeFile(&pathPlatform[0], buf[:])
}

func readRuntimeFile(path *byte, buf []byte) (uintptr, bool) {
	fd := sysOpen(path, openReadOnly, 0)
	if fd == sysErr {
		return 0, false
	}
	n := sysRead(fd, &buf[0], uintptr(len(buf)))
	if sysClose(fd) == sysErr || n == sysErr {
		return 0, false
	}
	return n, true
}

func writeRuntimeFile(path *byte, data []byte) bool {
	fd := sysOpen(path, openReadWrite, 0)
	if fd == sysErr {
		return false
	}
	if sysWrite(fd, &data[0], uintptr(len(data))) != uintptr(len(data)) {
		sysClose(fd)
		return false
	}
	if sysFsync(fd) == sysErr || sysClose(fd) == sysErr {
		return false
	}
	return true
}

func encodePkgState(state *pkgState, out *[pkgStateSize]byte) {
	out[0] = pkgMagicP
	out[1] = pkgMagicS
	out[2] = pkgMagicT
	out[3] = pkgMagic1
	out[4] = state.Phase
	out[5] = state.TrustedKey
	out[6] = state.BuildSeq
	out[7] = state.StableSeq
	out[8] = state.RollbackSeq
	out[9] = state.CatalogCount
	out[10] = state.InstalledMask
	out[11] = state.TelemetryCount
	copyPrefix8(out[12:20], state.MetadataDigest)
	copyPrefix8(out[20:28], state.CatalogDigest)
}

func decodePkgState(buf []byte, n uintptr, out *pkgState) bool {
	if int(n) != pkgStateSize || buf[0] != pkgMagicP || buf[1] != pkgMagicS || buf[2] != pkgMagicT || buf[3] != pkgMagic1 {
		return false
	}
	out.Phase = buf[4]
	out.TrustedKey = buf[5]
	out.BuildSeq = buf[6]
	out.StableSeq = buf[7]
	out.RollbackSeq = buf[8]
	out.CatalogCount = buf[9]
	out.InstalledMask = buf[10]
	out.TelemetryCount = buf[11]
	loadPrefix8(buf[12:20], &out.MetadataDigest)
	loadPrefix8(buf[20:28], &out.CatalogDigest)
	return true
}

func encodePlatformState(state *platformState, out *[platformStateSize]byte) {
	out[0] = platformMagicP
	out[1] = platformMagicL
	out[2] = platformMagicT
	out[3] = platformMagic1
	out[4] = state.SnapshotSeq
	out[5] = state.ResizeUnits
	out[6] = state.XattrLen
	out[7] = state.ReflinkCount
	out[8] = state.CapabilityMask
	out[9] = state.InstalledMask
	copyPrefix8(out[10:18], state.SnapshotDigest)
	copyPrefix8(out[18:26], state.XattrDigest)
	out[26] = 0
	out[27] = 0
}

func decodePlatformState(buf []byte, n uintptr, out *platformState) bool {
	if int(n) != platformStateSize || buf[0] != platformMagicP || buf[1] != platformMagicL || buf[2] != platformMagicT || buf[3] != platformMagic1 {
		return false
	}
	out.SnapshotSeq = buf[4]
	out.ResizeUnits = buf[5]
	out.XattrLen = buf[6]
	out.ReflinkCount = buf[7]
	out.CapabilityMask = buf[8]
	out.InstalledMask = buf[9]
	loadPrefix8(buf[10:18], &out.SnapshotDigest)
	loadPrefix8(buf[18:26], &out.XattrDigest)
	return true
}

func copyPrefix8(dst []byte, src [8]byte) {
	for idx := 0; idx < 8; idx++ {
		dst[idx] = src[idx]
	}
}

func loadPrefix8(src []byte, dst *[8]byte) {
	for idx := 0; idx < 8; idx++ {
		dst[idx] = src[idx]
	}
}

func pkgStateEqual(left *pkgState, right *pkgState) bool {
	return left.Phase == right.Phase &&
		left.TrustedKey == right.TrustedKey &&
		left.BuildSeq == right.BuildSeq &&
		left.StableSeq == right.StableSeq &&
		left.RollbackSeq == right.RollbackSeq &&
		left.CatalogCount == right.CatalogCount &&
		left.InstalledMask == right.InstalledMask &&
		left.TelemetryCount == right.TelemetryCount &&
		prefix8Equal(left.MetadataDigest, right.MetadataDigest) &&
		prefix8Equal(left.CatalogDigest, right.CatalogDigest)
}

func platformStateEqual(left *platformState, right *platformState) bool {
	return left.SnapshotSeq == right.SnapshotSeq &&
		left.ResizeUnits == right.ResizeUnits &&
		left.XattrLen == right.XattrLen &&
		left.ReflinkCount == right.ReflinkCount &&
		left.CapabilityMask == right.CapabilityMask &&
		left.InstalledMask == right.InstalledMask &&
		prefix8Equal(left.SnapshotDigest, right.SnapshotDigest) &&
		prefix8Equal(left.XattrDigest, right.XattrDigest)
}

func prefix8Equal(left [8]byte, right [8]byte) bool {
	for idx := 0; idx < 8; idx++ {
		if left[idx] != right[idx] {
			return false
		}
	}
	return true
}
