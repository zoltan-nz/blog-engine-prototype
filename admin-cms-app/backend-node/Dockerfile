FROM node:25-alpine

# Add pnpm
RUN npm config set fund false && npm upgrade -g npm && npm install -g pnpm && pnpm --version

WORKDIR /app

COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY . .

EXPOSE 8080

CMD ["pnpm", "dev"]

