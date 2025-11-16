# LLM Cost Ops Documentation

This is the documentation site for LLM Cost Ops, built using [Docusaurus](https://docusaurus.io/), a modern static website generator.

## Quick Start

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Access at http://localhost:3000
```

## Installation

```bash
npm install
```

Or using Yarn:

```bash
yarn install
```

## Available Scripts

### Development

```bash
# Start development server with hot reload
npm run dev

# Or use the standard Docusaurus command
npm start
```

This command starts a local development server and opens up a browser window. Most changes are reflected live without having to restart the server.

### Production Build

```bash
# Build the documentation site
npm run build

# Serve the production build locally
npm run serve
```

This command generates static content into the `build` directory and can be served using any static contents hosting service.

### Code Quality

```bash
# Type checking
npm run typecheck

# Lint code
npm run lint

# Fix linting issues
npm run lint:fix

# Format code
npm run format

# Check formatting
npm run format:check

# Pre-commit checks
npm run precommit
```

### Testing

```bash
# Run all tests (build + link checking)
npm run test

# Check links (requires local server running)
npm run check-links

# Check external links
npm run check-links:external
```

### Deployment

```bash
# Deploy to GitHub Pages
npm run deploy

# Deploy using GitHub Actions (recommended)
# Just push to main branch - automatic deployment via CI/CD
```

## Project Structure

```
website/
├── blog/                  # Blog posts
├── docs/                  # Documentation markdown files
│   ├── intro.md
│   ├── api/              # API documentation
│   ├── sdks/             # SDK documentation
│   └── ...
├── src/                   # React components and pages
│   ├── components/
│   ├── css/
│   └── pages/
├── static/                # Static assets
│   └── img/
├── .github/workflows/     # CI/CD workflows
├── docusaurus.config.ts   # Docusaurus configuration
├── sidebars.ts           # Sidebar configuration
├── package.json
└── README.md
```

## Configuration

### Environment Variables

Copy `.env.example` to `.env.local` and customize:

```bash
cp .env.example .env.local
```

See [DEPLOYMENT.md](./DEPLOYMENT.md) for detailed configuration options.

### Docusaurus Configuration

Main configuration is in `docusaurus.config.ts`. Key settings:

- **Site metadata**: Title, tagline, URL
- **Deployment**: Organization name, project name
- **Theme**: Colors, navbar, footer
- **Plugins**: Search, analytics, etc.

### Sidebar Configuration

Sidebar navigation is configured in `sidebars.ts`.

## Deployment Options

This documentation site supports multiple deployment platforms:

### 1. GitHub Pages (Recommended)

Automatic deployment via GitHub Actions:
- Push to `main` branch
- Workflow builds and deploys automatically
- Access at: `https://<org>.github.io/<repo>/`

### 2. Netlify

Features:
- Automatic deployments from Git
- Deploy previews for PRs
- Custom domains
- Forms and serverless functions

Configuration in `netlify.toml`.

### 3. Vercel

Features:
- Automatic deployments
- Preview deployments
- Edge network
- Analytics

Configuration in `vercel.json`.

### 4. Docker

Build and run as a containerized application:

```bash
# Development
docker-compose up docs-dev

# Production
docker-compose up docs-prod
```

For detailed deployment instructions, see [DEPLOYMENT.md](./DEPLOYMENT.md).

## Development Workflow

1. **Create a new branch**
   ```bash
   git checkout -b feature/your-feature
   ```

2. **Make changes**
   - Edit documentation in `docs/`
   - Add blog posts in `blog/`
   - Update components in `src/`

3. **Test locally**
   ```bash
   npm run dev
   npm run typecheck
   npm run lint
   ```

4. **Commit and push**
   ```bash
   git add .
   git commit -m "Description of changes"
   git push origin feature/your-feature
   ```

5. **Create Pull Request**
   - Automatic preview deployment on Netlify
   - Review in preview environment
   - Merge when approved

6. **Automatic deployment**
   - Merging to `main` triggers production deployment
   - Site updates automatically

## Writing Documentation

### Creating New Pages

Create a new markdown file in `docs/`:

```markdown
---
id: unique-id
title: Page Title
sidebar_label: Sidebar Label
---

# Page Title

Your content here...
```

### Adding to Sidebar

Edit `sidebars.ts` to add your page to navigation.

### Markdown Features

Docusaurus supports:
- Standard Markdown
- MDX (JSX in Markdown)
- Code blocks with syntax highlighting
- Admonitions (notes, warnings, tips)
- Tabs
- Live code editors

Example:

```markdown
:::tip
This is a helpful tip!
:::

```typescript
// Code block with syntax highlighting
const example = "Hello, World!";
```
```

### API Documentation

API documentation can be auto-generated using TypeDoc plugin.

## Link Checking

Ensure all links are valid:

```bash
# Start local server
npm run serve

# In another terminal, check links
npm run check-links
```

Configuration in `.lycheerc.toml`.

## Performance

### Optimization Tips

1. **Images**: Use optimized formats (WebP, AVIF)
2. **Code splitting**: Automatic with Docusaurus
3. **Lazy loading**: Images and components
4. **CDN**: Enabled by default on Netlify/Vercel
5. **Compression**: Gzip/Brotli enabled

### Lighthouse Audits

Automatic Lighthouse CI runs on every deployment:

```bash
npm install -g @lhci/cli
lhci autorun
```

## Troubleshooting

### Build Issues

```bash
# Clear cache and rebuild
npm run clear
rm -rf node_modules package-lock.json
npm install
npm run build
```

### Port Already in Use

```bash
# Kill process on port 3000
npx kill-port 3000

# Or use a different port
npm start -- --port 3001
```

### Link Checker Fails

Some external links may be rate-limited. Add exclusions to `.lycheerc.toml`.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed guidelines.

## Resources

- [Docusaurus Documentation](https://docusaurus.io/docs)
- [Markdown Guide](https://www.markdownguide.org/)
- [MDX Documentation](https://mdxjs.com/)
- [Deployment Guide](./DEPLOYMENT.md)

## License

This documentation is part of the LLM Cost Ops project and is licensed under the Apache License 2.0. See [LICENSE](../LICENSE) for details.

## Support

- **Documentation Issues**: Open an issue in the repository
- **Questions**: Join our [Discord](https://discord.gg/llm-cost-ops)
- **Email**: support@llm-cost-ops.dev
