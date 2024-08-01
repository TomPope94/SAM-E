#!/usr/bin/env bash

minio server /buckets & \
  server_pid=$!; \
  until mc alias set local http://localhost:9000 minioadmin minioadmin; do \
      sleep 1; \
  done; \

  {%- for infra in infrastructure %}
    {%- if infra.infrastructure_type == "S3" %}
  mc mb local/{{infra.Properties.Name}}; \
  mc anonymous set public local/{{infra.Properties.Name}}; \
  mc event add local/{{infra.Properties.Name}} arn:minio:sqs::LOCAL:webhook --event put,get,delete; \
    {%- endif %}
  {%- endfor %}

  kill $server_pid;

minio server /buckets --address :9000 --console-address :9001;
