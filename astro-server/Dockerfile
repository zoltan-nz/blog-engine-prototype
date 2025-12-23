FROM node:25-alpine

# Add pnpm
RUN npm config set fund false && npm upgrade -g npm && npm install -g pnpm && pnpm --version

WORKDIR /app

RUN mkdir -p /app/sites

CMD ["tail", "-f", "/dev/null"]

