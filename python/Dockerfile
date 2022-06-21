FROM python:3.9-slim-buster
WORKDIR /app
COPY ./ ./
RUN pip3 install flask 
RUN export PATH=$PATH:/home/temp/.local/bin
ENTRYPOINT ["python3", "app.py"]