# syntax=docker/dockerfile:1
FROM node:26-alpine AS builder
WORKDIR /console
COPY apps/console/package.json apps/console/pnpm-lock.yaml apps/console/pnpm-workspace.yaml ./
RUN npm install --global pnpm@11.0.5 && pnpm install --frozen-lockfile
COPY apps/console ./
RUN pnpm build && mkdir -p /out && cp -R dist/. /out/

FROM nginx:alpine AS runtime
COPY --from=builder /out /usr/share/nginx/html
EXPOSE 80
