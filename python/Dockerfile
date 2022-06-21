FROM python:3.9-slim-buster
RUN useradd -ms /bin/bash temp
USER temp
WORKDIR /app
COPY ./ ./
RUN pip3 install flask 
RUN export PATH=$PATH:/home/temp/.local/bin
ENTRYPOINT ["python3", "app.py"]