FROM node:20-slim
WORKDIR /app
COPY package.json .
RUN npm install --omit=dev
COPY consumer.js .
ENTRYPOINT ["node", "consumer.js"]
