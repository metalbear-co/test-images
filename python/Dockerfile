FROM python:3.9-slim-buster
WORKDIR /app
COPY app.py ../test.txt .
RUN pip3 install flask 
ENTRYPOINT ["python3", "app.py"]