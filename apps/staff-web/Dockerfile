FROM node:22-bookworm-slim AS deps

WORKDIR /workspace

COPY package.json package-lock.json ./
COPY apps/staff-web/package.json ./apps/staff-web/package.json
RUN npm ci --workspace @pet-resort/staff-web --include-workspace-root

FROM node:22-bookworm-slim AS builder

WORKDIR /workspace
ENV NEXT_TELEMETRY_DISABLED=1

COPY --from=deps /workspace/node_modules ./node_modules
COPY package.json package-lock.json ./
COPY apps/staff-web ./apps/staff-web

RUN npm --workspace @pet-resort/staff-web run build

FROM node:22-bookworm-slim AS runtime

WORKDIR /app
ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1
ENV HOSTNAME=0.0.0.0
ENV PORT=3000

RUN useradd --system --user-group --home-dir /nonexistent --shell /usr/sbin/nologin nextjs

COPY --from=builder --chown=nextjs:nextjs /workspace/apps/staff-web/.next/standalone ./
COPY --from=builder --chown=nextjs:nextjs /workspace/apps/staff-web/.next/static ./apps/staff-web/.next/static

USER nextjs
EXPOSE 3000

HEALTHCHECK --interval=10s --timeout=3s --retries=12 --start-period=10s \
  CMD node -e "fetch('http://127.0.0.1:3000/').then((r)=>process.exit(r.ok?0:1)).catch(()=>process.exit(1))"

CMD ["node", "apps/staff-web/server.js"]
