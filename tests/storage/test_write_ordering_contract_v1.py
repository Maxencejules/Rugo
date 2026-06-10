"""M13 acceptance: write ordering/barrier contract checks."""

from pathlib import Path
import sys

sys.path.append(str(Path(__file__).resolve().parent))
from storage_v1_model import WriteOrderingModel  # noqa: E402


def test_ordered_commit_sequence_is_valid():
    m = WriteOrderingModel()
    m.write_data()
    m.data_barrier()
    m.write_metadata()
    m.metadata_barrier()
    m.write_clean_marker()

    assert m.ordering_valid is True
    assert m.clean_marker_written is True


def test_metadata_before_data_barrier_is_invalid():
    m = WriteOrderingModel()
    m.write_data()
    m.write_metadata()
    m.metadata_barrier()
    m.write_clean_marker()

    assert m.ordering_valid is False
