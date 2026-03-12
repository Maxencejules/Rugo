"""Deterministic reference model for M54 native storage durability semantics."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class NativeStorageDurabilityModel:
    controller: str
    fua_supported: bool
    data_pending: bool = False
    metadata_pending: bool = False
    data_durable: bool = False
    metadata_durable: bool = False
    cache_dirty: bool = False

    def write_data(self) -> None:
        self.data_pending = True
        self.cache_dirty = True

    def write_metadata(self) -> None:
        self.metadata_pending = True
        self.cache_dirty = True

    def _commit_pending(self) -> None:
        if self.data_pending:
            self.data_durable = True
            self.data_pending = False
        if self.metadata_pending:
            self.metadata_durable = True
            self.metadata_pending = False
        self.cache_dirty = False

    def fua_commit(self) -> None:
        if not self.fua_supported:
            raise RuntimeError(f"{self.controller} does not support FUA in this model")
        self._commit_pending()

    def cache_flush(self) -> None:
        self._commit_pending()

    def crash(self) -> tuple[bool, bool]:
        return self.data_durable, self.metadata_durable
