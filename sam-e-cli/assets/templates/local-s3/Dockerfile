FROM minio/minio:latest

COPY --from=minio/mc:latest /usr/bin/mc /usr/bin/mc

ADD entrypoint.sh /local-s3/entrypoint.sh
RUN chmod +x /local-s3/entrypoint.sh

ENTRYPOINT ["/local-s3/entrypoint.sh"]
