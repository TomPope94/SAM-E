FROM public.ecr.aws/docker/library/python:3.12-slim

# Copy Lambda adapter extension
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.9.1 /lambda-adapter /opt/extensions/lambda-adapter
RUN chmod +x /opt/extensions/lambda-adapter

# Install uv
COPY --from=ghcr.io/astral-sh/uv:latest /uv /usr/local/bin/uv

ENV PORT=8000
WORKDIR /var/task

# Copy dependency files first for better caching
COPY pyproject.toml uv.lock ./

# Install dependencies
RUN uv sync --frozen --no-dev

# Copy application code
COPY *.py ./

# Create startup script that mimics Lambda extension initialization
RUN echo '#!/bin/bash\n\
/opt/extensions/lambda-adapter &\n\
exec uv run uvicorn --port=$PORT main:app\n\
' > /start.sh && chmod +x /start.sh

# Start extension in background, then start the application
CMD ["/start.sh"]
