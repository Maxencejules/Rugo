"""M19 acceptance: DNS-stub v2 behavior and diagnostics artifacts."""

import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]

sys.path.append(str(Path(__file__).resolve().parent))
from v2_model import DnsStubV2Model  # noqa: E402

sys.path.append(str(ROOT / "tools"))
import run_net_interop_matrix_v2 as interop  # noqa: E402
import run_net_soak_v2 as soak  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_socket_contract_v2_declares_dns_stub_behavior():
    text = _read("docs/net/socket_contract_v2.md")
    for token in [
        "## DNS stub behavior",
        "`A`",
        "`AAAA`",
        "deterministic `NXDOMAIN`",
        "TTL-bounded cache behavior",
    ]:
        assert token in text


def test_dns_stub_model_answers_a_aaaa_and_nxdomain():
    model = DnsStubV2Model()

    rc, records, ttl = model.query("svc.rugo.local", "A")
    assert rc == 0
    assert records == ["10.0.2.15"]
    assert ttl == 30

    rc6, records6, ttl6 = model.query("svc.rugo.local", "AAAA")
    assert rc6 == 0
    assert records6 == ["2001:db8::15"]
    assert ttl6 == 30

    model.tick(31)
    rc_after, records_after, ttl_after = model.query("svc.rugo.local", "A")
    assert rc_after == 0
    assert records_after == ["10.0.2.15"]
    assert ttl_after == 30

    rc_missing, records_missing, ttl_missing = model.query("missing.rugo.local", "A")
    assert rc_missing == -2
    assert records_missing == []
    assert ttl_missing == 0


def test_interop_and_soak_v2_tools_emit_expected_schema(tmp_path: Path):
    interop_out = tmp_path / "net-interop-v2.json"
    soak_out = tmp_path / "net-soak-v2.json"

    assert interop.main(["--out", str(interop_out), "--target-pass-rate", "0.95"]) == 0
    assert soak.main(["--seed", "20260308", "--iterations", "800", "--out", str(soak_out)]) == 0

    interop_data = json.loads(interop_out.read_text(encoding="utf-8"))
    assert interop_data["schema"] == "rugo.net_interop_matrix.v2"
    assert interop_data["total_cases"] >= 9
    assert interop_data["pass_rate"] == 1.0
    assert interop_data["meets_target"] is True

    soak_data = json.loads(soak_out.read_text(encoding="utf-8"))
    assert soak_data["schema"] == "rugo.net_soak_report.v2"
    assert soak_data["iterations"] == 800
    assert soak_data["total_failures"] == 0
    assert soak_data["meets_target"] is True
