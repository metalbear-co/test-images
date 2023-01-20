FROM node:buster
WORKDIR /app
RUN npm install ws
COPY app.mjs .
ENTRYPOINT ["node", "app.mjs"]