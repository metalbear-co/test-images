FROM node:buster
WORKDIR /app
RUN npm install express
COPY app.js test.txt .
ENTRYPOINT ["node", "app.js"]
