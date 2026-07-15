from __future__ import annotations

from pathlib import Path

from minio import Minio


class ObjectStorage:
    """MinIO / S3 storage adapter."""

    def __init__(
        self,
        endpoint: str,
        access_key: str,
        secret_key: str,
        secure: bool = False,
    ) -> None:
        self._client = Minio(
            endpoint,
            access_key=access_key,
            secret_key=secret_key,
            secure=secure,
        )

    def ensure_bucket(self, bucket: str) -> None:
        if not self._client.bucket_exists(bucket):
            self._client.make_bucket(bucket)

    def upload_file(
        self,
        bucket: str,
        object_name: str,
        file_path: str | Path,
    ) -> None:
        self._client.fput_object(
            bucket,
            object_name,
            str(file_path),
        )

    def download_file(
        self,
        bucket: str,
        object_name: str,
        destination: str | Path,
    ) -> None:
        self._client.fget_object(
            bucket,
            object_name,
            str(destination),
        )

    def remove_object(
        self,
        bucket: str,
        object_name: str,
    ) -> None:
        self._client.remove_object(bucket, object_name)