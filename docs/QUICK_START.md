# Documentation Quick Start Guide

Get the LLM Cost Ops documentation site running in 5 minutes.

## Prerequisites

- Node.js 20 or higher
- npm (comes with Node.js)
- Git

## Local Development

### 1. Navigate to website directory
```bash
cd website
```

### 2. Install dependencies
```bash
npm install
```

### 3. Start development server
```bash
npm run dev
```

The site will open at http://localhost:3000 with hot reload enabled.

## Quick Commands

### Development
```bash
npm run dev              # Start dev server (recommended)
npm start                # Alternative start command
```

### Building
```bash
npm run build           # Build production site
npm run serve           # Serve production build locally
```

### Code Quality
```bash
npm run typecheck       # Check TypeScript types
npm run lint            # Lint code
npm run format:check    # Check code formatting
```

## Making Changes

### 1. Edit documentation
- Markdown files are in `docs/` directory
- Blog posts are in `blog/` directory
- Edit and save - changes appear instantly

### 2. Add new pages
Create a new markdown file in `docs/`:
```markdown
---
id: my-page
title: My Page Title
---

# Content here
```

### 3. Update navigation
Edit `sidebars.ts` to add your page to the sidebar.

## Deployment Options

### GitHub Pages (Automatic)
```bash
# Just push to main branch
git add .
git commit -m "Update docs"
git push origin main

# GitHub Actions automatically deploys
```

### Manual Deploy
```bash
npm run build
npm run deploy
```

### Docker (Production)
```bash
# Build and run with Docker Compose
docker-compose up docs-prod

# Access at http://localhost:8080
```

### Docker (Development)
```bash
# Run development mode with hot reload
docker-compose up docs-dev

# Access at http://localhost:3000
```

## Environment Setup (Optional)

Create `.env.local` for custom configuration:
```bash
cp .env.example .env.local
```

Edit `.env.local` with your settings:
- Analytics IDs
- Search configuration
- Feature flags

## Testing

### Build Test
```bash
npm run build
npm run serve
```

### Link Check
```bash
# In one terminal
npm run serve

# In another terminal
npm run check-links
```

## Troubleshooting

### Port Already in Use
```bash
# Kill process on port 3000
npx kill-port 3000

# Or use different port
npm start -- --port 3001
```

### Build Errors
```bash
# Clear cache and rebuild
npm run clear
rm -rf node_modules package-lock.json
npm install
npm run build
```

### Module Not Found
```bash
# Reinstall dependencies
rm -rf node_modules
npm install
```

## Next Steps

1. **Read the full guide:** See [README.md](./README.md)
2. **Deployment details:** See [DEPLOYMENT.md](./DEPLOYMENT.md)
3. **Edit content:** Start adding your documentation
4. **Configure deployment:** Set up GitHub Pages, Netlify, or Vercel

## Common Workflows

### Adding a New Doc Page
```bash
# 1. Create the file
touch docs/my-new-doc.md

# 2. Add content
cat > docs/my-new-doc.md << EOF
---
id: my-new-doc
title: My New Documentation
---

# My New Documentation

Content here...
EOF

# 3. Add to sidebar
# Edit sidebars.ts and add 'my-new-doc' to appropriate section

# 4. View changes
npm run dev
```

### Creating a Blog Post
```bash
# 1. Create the file
touch blog/2024-11-16-my-post.md

# 2. Add content with frontmatter
cat > blog/2024-11-16-my-post.md << EOF
---
slug: my-first-post
title: My First Blog Post
authors: [yourname]
tags: [tutorial, documentation]
---

# My First Post

Post content here...
EOF

# 3. View at http://localhost:3000/blog
```

### Deploying to Production
```bash
# 1. Ensure everything builds
npm run build

# 2. Run quality checks
npm run typecheck
npm run lint
npm run format:check

# 3. Commit and push
git add .
git commit -m "Deploy updates"
git push origin main

# 4. Check GitHub Actions
# Go to repository > Actions tab
# Watch deployment progress
```

## Support

- **Documentation:** [README.md](./README.md) and [DEPLOYMENT.md](./DEPLOYMENT.md)
- **Issues:** Open an issue on GitHub
- **Community:** Join our Discord
- **Docusaurus Docs:** https://docusaurus.io/docs

## Tips

- Save files frequently - hot reload is instant
- Use Prettier to format code automatically
- Check the browser console for errors
- Test production build before deploying
- Use preview deployments for big changes
