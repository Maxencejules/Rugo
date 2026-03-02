"""R4 acceptance test: service registry rejects inactive endpoint IDs."""


def test_svc_bad_endpoint(qemu_serial_svc_bad_endpoint):
    """sys_svc_register must return -1 when endpoint is not active."""
    out = qemu_serial_svc_bad_endpoint.stdout
    assert "SVC: bad endpoint ok" in out, f"Missing 'SVC: bad endpoint ok'. Got:\n{out}"
