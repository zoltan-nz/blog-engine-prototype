FROM node:25-alpine

RUN npm config set fund false && npm upgrade -g npm && npm install -g pnpm && pnpm --version

# git is needed for site creation (init, commit, push)
RUN apk add --no-cache git

WORKDIR /app

RUN mkdir -p /app/astro-sites /app/git-repos

COPY management-api.mjs ./

EXPOSE 4320

CMD ["node", "management-api.mjs"]
