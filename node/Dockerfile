FROM node:buster
RUN npm install -g express
WORKDIR /app
COPY node/app.js .
ENTRYPOINT ["node", "app.js"]