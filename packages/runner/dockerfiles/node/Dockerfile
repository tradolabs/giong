FROM node:26-bookworm-slim
RUN npm install --global pnpm@12.15.0
WORKDIR /workspace
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile
COPY . .
CMD ["node", "main.ts"]
