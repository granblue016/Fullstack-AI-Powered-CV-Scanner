# Build stage
FROM node:18-alpine as builder

WORKDIR /app

# Copy package files
COPY package.json package-lock.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY . .

# Build
RUN npm run build

# Runtime stage
FROM node:18-alpine

WORKDIR /app

# Install only production dependencies
COPY package.json package-lock.json ./
RUN npm ci --only=production

# Copy built application from builder
COPY --from=builder /app/dist ./dist

ENV PORT=3000
EXPOSE 3000

# Serve the static build
CMD ["npm", "run", "preview"]
