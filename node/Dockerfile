FROM node:buster
RUN npm install -g express
WORKDIR /app
COPY app.js .
ENTRYPOINT ["node", "app.js"]