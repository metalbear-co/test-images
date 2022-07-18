FROM node:buster
RUN npm install -g express
WORKDIR /app
COPY app.js ../test.txt .
ENTRYPOINT ["node", "app.js"]