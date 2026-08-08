FROM python:3.12-alpine

LABEL org.opencontainers.image.title="NVA information-lifespan Hermes processor"
LABEL org.opencontainers.image.description="Deterministic local bridge for synthetic Manager Daily Report trace processing"

WORKDIR /app
COPY apps/hermes-processor/processor.py /app/processor.py

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    HERMES_PROCESSOR_MODE=deterministic_local_bridge \
    HERMES_PROCESSOR_REQUIRE_RUNTIME=false \
    HERMES_PROCESSOR_INPUT=/workspace/input/hermes-processor-input.json \
    HERMES_PROCESSOR_OUTPUT_DIR=/workspace/output

ENTRYPOINT ["python", "/app/processor.py"]
