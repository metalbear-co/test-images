FROM node:buster
WORKDIR /app
RUN npm install express
COPY app.js .
ENTRYPOINT ["node", "app.js"]
