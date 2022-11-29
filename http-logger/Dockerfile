FROM python:3.9-slim-buster
WORKDIR /app
COPY http_logger.py .
RUN pip3 install flask 
ENTRYPOINT ["python3", "http_logger.py"]
