FROM minio/minio:latest

COPY --from=minio/mc:latest /usr/bin/mc /usr/bin/mc
RUN mkdir /buckets
RUN minio server /buckets & \
    server_pid=$!; \
    until mc alias set local http://localhost:9000 minioadmin minioadmin; do \
        sleep 1; \
    done; \
    {%- for infra in infrastructure %}
    {%- if infra.infrastructure_type == "S3" %}
    mc mb local/{{infra.name}}; \
    {%- endif %}
    {%- endfor %}
    kill $server_pid

CMD ["minio", "server", "/buckets", "--address", ":9000", "--console-address", ":9001"]
