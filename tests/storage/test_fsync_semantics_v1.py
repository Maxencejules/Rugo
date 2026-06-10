"""M13 acceptance: durability and fsync semantic baseline."""

from pathlib import Path
import sys

sys.path.append(str(Path(__file__).resolve().parent))
from storage_v1_model import DurabilityModel  # noqa: E402


def test_fdatasync_persists_data_not_metadata():
    model = DurabilityModel()
    model.write_data()
    model.write_metadata()
    model.fdatasync()

    data_ok, meta_ok = model.crash()
    assert data_ok is True
    assert meta_ok is False


def test_fsync_persists_data_and_metadata():
    model = DurabilityModel()
    model.write_data()
    model.write_metadata()
    model.fsync()

    data_ok, meta_ok = model.crash()
    assert data_ok is True
    assert meta_ok is True
