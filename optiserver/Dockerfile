FROM python:3.11-slim
WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY optimization_core.py .
COPY server.py .

EXPOSE 8501
CMD ["python", "-m", "uvicorn", "server:app", "--host", "0.0.0.0", "--port", "8501"]