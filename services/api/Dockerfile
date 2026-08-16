FROM python:3.12-slim AS runtime

WORKDIR /opt/lawsynth
COPY python/lawsynth-server /opt/lawsynth/python/lawsynth-server
COPY services/api /opt/lawsynth/services/api
RUN pip install --no-cache-dir \
    /opt/lawsynth/python/lawsynth-server \
    "/opt/lawsynth/services/api[server]" \
    && mkdir -p /var/lib/lawsynth/objects \
    && chown -R 65532:65532 /var/lib/lawsynth

ENV PYTHONUNBUFFERED=1 \
    LAWSYNTH_API_ENV=production \
    LAWSYNTH_DATABASE_URL=sqlite:////var/lib/lawsynth/metadata.sqlite3 \
    LAWSYNTH_OBJECT_ROOT=/var/lib/lawsynth/objects
USER 65532:65532
EXPOSE 8080
CMD ["gunicorn", "--bind", "0.0.0.0:8080", "--workers", "1", "--access-logfile", "-", "--error-logfile", "-", "lawsynth_api.main:application"]
