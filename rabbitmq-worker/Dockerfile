# --- base ---

FROM node:25-slim as base
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN npm install --global pnpm@latest-10

# --- build ---

FROM base AS build
WORKDIR /app

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml .

RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile

COPY . .

RUN pnpm build

# --- prod ---

FROM base

WORKDIR /app

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml .
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --prod

COPY --from=build /app/dist ./dist

EXPOSE 3000

CMD ["pnpm", "start:prod"]
