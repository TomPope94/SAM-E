FROM minio/minio:latest

COPY --from=minio/mc:latest /usr/bin/mc /usr/bin/mc
RUN mkdir /buckets
RUN minio server /buckets & \
    server_pid=$!; \
    until mc alias set local http://localhost:9000 minioadmin minioadmin; do \
        sleep 1; \
    done; \
    mc mb local/bucket1; \
    mc mb local/bucket_test; \
    mc mb local/another_bucket; \
    kill $server_pid

CMD ["minio", "server", "/buckets", "--address", ":9000", "--console-address", ":9001"]
