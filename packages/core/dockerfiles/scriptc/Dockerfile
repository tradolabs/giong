FROM node:26-bookworm-slim
RUN apt-get update && apt-get install -y clang
RUN npm install --global scriptc@0.4.1
WORKDIR /workspace
COPY . .
CMD ["scriptc", "run", "main.ts"]
