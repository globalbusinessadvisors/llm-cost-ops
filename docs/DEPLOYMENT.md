# Documentation Deployment Guide

This guide covers all deployment options for the LLM Cost Ops documentation site.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Deployment Options](#deployment-options)
  - [GitHub Pages](#github-pages)
  - [Netlify](#netlify)
  - [Vercel](#vercel)
  - [Docker](#docker)
- [CI/CD Pipeline](#cicd-pipeline)
- [Environment Variables](#environment-variables)
- [Monitoring and Maintenance](#monitoring-and-maintenance)

## Prerequisites

- Node.js 20 or higher
- npm or yarn
- Git
- (Optional) Docker for containerized deployment

## Deployment Options

### GitHub Pages

GitHub Pages provides free hosting for static sites directly from your repository.

#### Setup

1. **Enable GitHub Pages in Repository Settings**
   - Go to repository Settings > Pages
   - Set Source to "GitHub Actions"
   - Save changes

2. **Configure Custom Domain (Optional)**
   - Add a CNAME record in your DNS settings:
     ```
     docs.llm-cost-ops.dev -> <username>.github.io
     ```
   - Add custom domain in GitHub Pages settings
   - Enable "Enforce HTTPS"

3. **Deploy**
   - Push to main branch
   - GitHub Actions workflow automatically builds and deploys
   - Check workflow status in Actions tab

#### Manual Deployment

```bash
cd website
npm run build
npm run deploy
```

This uses the built-in Docusaurus deployment command configured for GitHub Pages.

### Netlify

Netlify offers automatic deployments, preview URLs for PRs, and advanced features.

#### Setup

1. **Connect Repository**
   - Sign up at [netlify.com](https://netlify.com)
   - Click "New site from Git"
   - Select your repository
   - Configure build settings:
     - Base directory: `website`
     - Build command: `npm run build`
     - Publish directory: `website/build`

2. **Add Environment Variables**
   - Go to Site Settings > Environment Variables
   - Add required variables (see [Environment Variables](#environment-variables))

3. **Configure Secrets in GitHub**
   ```bash
   # Get your Netlify site ID and auth token from Netlify dashboard
   gh secret set NETLIFY_SITE_ID
   gh secret set NETLIFY_AUTH_TOKEN
   ```

4. **Custom Domain**
   - Go to Site Settings > Domain Management
   - Add custom domain: `docs.llm-cost-ops.dev`
   - Follow DNS configuration instructions
   - Enable HTTPS (automatic with Let's Encrypt)

#### Manual Deployment

```bash
# Install Netlify CLI
npm install -g netlify-cli

# Login to Netlify
netlify login

# Deploy
cd website
npm run build
netlify deploy --prod --dir=build
```

#### Deploy Previews

- Every PR automatically gets a preview deployment
- Preview URL is posted as a comment on the PR
- Accessible at: `https://deploy-preview-{PR-number}--llm-cost-ops.netlify.app`

### Vercel

Vercel offers excellent performance and automatic deployments.

#### Setup

1. **Connect Repository**
   - Sign up at [vercel.com](https://vercel.com)
   - Import your repository
   - Configure project:
     - Framework Preset: Docusaurus
     - Root Directory: `website`
     - Build Command: `npm run build`
     - Output Directory: `build`

2. **Add Environment Variables**
   - Go to Project Settings > Environment Variables
   - Add required variables

3. **Custom Domain**
   - Go to Project Settings > Domains
   - Add `docs.llm-cost-ops.dev`
   - Follow DNS configuration instructions

#### Manual Deployment

```bash
# Install Vercel CLI
npm install -g vercel

# Login to Vercel
vercel login

# Deploy
cd website
vercel --prod
```

### Docker

Deploy as a containerized application using Docker and Nginx.

#### Build Docker Image

```bash
cd website

# Build the image
docker build -t llm-cost-ops-docs:latest .

# Run the container
docker run -d \
  --name llm-cost-ops-docs \
  -p 8080:8080 \
  --restart unless-stopped \
  llm-cost-ops-docs:latest
```

#### Using Docker Compose

**Development Mode:**
```bash
cd website
docker-compose up docs-dev
```

Access at http://localhost:3000 with hot reload.

**Production Mode:**
```bash
cd website
docker-compose up -d docs-prod
```

Access at http://localhost:8080.

#### Kubernetes Deployment

Create a deployment manifest:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-cost-ops-docs
  labels:
    app: llm-cost-ops-docs
spec:
  replicas: 3
  selector:
    matchLabels:
      app: llm-cost-ops-docs
  template:
    metadata:
      labels:
        app: llm-cost-ops-docs
    spec:
      containers:
      - name: docs
        image: ghcr.io/llm-cost-ops/llm-cost-ops-docs:latest
        ports:
        - containerPort: 8080
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: llm-cost-ops-docs
spec:
  selector:
    app: llm-cost-ops-docs
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
```

Deploy:
```bash
kubectl apply -f k8s-deployment.yml
```

## CI/CD Pipeline

The project includes a comprehensive GitHub Actions workflow (`.github/workflows/docs-deploy.yml`) that:

### On Pull Requests:
1. Validates documentation
2. Type checks
3. Lints code
4. Builds documentation
5. Deploys preview to Netlify
6. Posts preview URL as PR comment

### On Push to Main:
1. Validates documentation
2. Builds production-ready site
3. Deploys to GitHub Pages
4. Deploys to Netlify (production)
5. Runs link checker
6. Performs Lighthouse performance audit

### Manual Trigger:
- Can be manually triggered via GitHub Actions UI
- Useful for re-deployments without code changes

## Environment Variables

Create a `.env.local` file in the `website` directory:

```bash
# Copy example environment file
cp .env.example .env.local
```

### Required Variables

- `NODE_ENV`: Environment (development/production)
- `DOCUSAURUS_CONFIG`: Configuration profile

### Optional Variables

- `GA_TRACKING_ID`: Google Analytics tracking ID
- `ALGOLIA_APP_ID`: Algolia search app ID
- `ALGOLIA_API_KEY`: Algolia search API key
- `ALGOLIA_INDEX_NAME`: Algolia search index name

### GitHub Secrets

For CI/CD, configure these secrets in your GitHub repository:

```bash
# Netlify
gh secret set NETLIFY_AUTH_TOKEN
gh secret set NETLIFY_SITE_ID

# Vercel (if using)
gh secret set VERCEL_TOKEN
gh secret set VERCEL_ORG_ID
gh secret set VERCEL_PROJECT_ID
```

## Local Development

```bash
cd website

# Install dependencies
npm install

# Start development server
npm run dev

# Access at http://localhost:3000
```

### Development Commands

```bash
# Start development server
npm run dev

# Build production site
npm run build

# Serve production build locally
npm run serve

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

# Check links
npm run check-links

# Run all tests
npm run test
```

## Testing Deployment

### Local Testing

1. **Build and serve locally:**
   ```bash
   npm run build
   npm run serve
   ```

2. **Test with Docker:**
   ```bash
   docker-compose up docs-prod
   ```

3. **Check links:**
   ```bash
   npm run check-links
   ```

### Preview Deployments

- Create a PR to get automatic preview deployment
- Check preview URL in PR comments
- Verify all links and functionality

## Link Checking

The project includes automated link checking:

### Lychee Link Checker

Configuration in `.lycheerc.toml`:

```bash
# Check all links in build directory
lychee --config .lycheerc.toml build/**/*.html
```

### Broken Link Checker

```bash
# Start server
npm run serve

# In another terminal
npm run check-links
```

## Performance Monitoring

### Lighthouse CI

Automatic Lighthouse audits run on every deployment to main:

- Performance metrics
- Accessibility scores
- Best practices
- SEO analysis

Results are available in GitHub Actions artifacts.

### Manual Lighthouse Audit

```bash
# Install Lighthouse
npm install -g @lhci/cli

# Run audit
lhci autorun --config=lighthouserc.js
```

## Troubleshooting

### Build Failures

**Problem:** Build fails with "Module not found"
```bash
# Solution: Clear cache and reinstall
rm -rf node_modules .docusaurus
npm install
npm run build
```

**Problem:** Type errors
```bash
# Solution: Run type checking
npm run typecheck
```

### Deployment Issues

**GitHub Pages not updating:**
- Check GitHub Actions status
- Verify gh-pages branch exists
- Check repository permissions

**Netlify deployment fails:**
- Check build logs in Netlify dashboard
- Verify environment variables
- Check base directory setting

**Vercel deployment fails:**
- Check deployment logs
- Verify vercel.json configuration
- Check build command and output directory

### Link Checking Failures

**External links failing:**
- Check `.lycheerc.toml` exclusions
- Some sites may block automated checkers
- Rate limiting may cause temporary failures

**Internal links failing:**
- Check file paths and extensions
- Verify sidebar configuration
- Check for typos in links

## Rollback Procedures

### GitHub Pages

```bash
# Revert to previous commit
git revert HEAD
git push origin main
```

### Netlify

- Go to Netlify dashboard
- Navigate to Deploys
- Click "..." on previous successful deploy
- Select "Publish deploy"

### Vercel

- Go to Vercel dashboard
- Navigate to Deployments
- Find previous successful deployment
- Click "Promote to Production"

### Docker

```bash
# Roll back to previous image
docker pull ghcr.io/llm-cost-ops/llm-cost-ops-docs:previous-tag
docker stop llm-cost-ops-docs
docker rm llm-cost-ops-docs
docker run -d --name llm-cost-ops-docs -p 8080:8080 \
  ghcr.io/llm-cost-ops/llm-cost-ops-docs:previous-tag
```

## Monitoring and Maintenance

### Uptime Monitoring

Recommended services:
- UptimeRobot
- Pingdom
- StatusCake

### Analytics

- Google Analytics for traffic
- Algolia Analytics for search
- Netlify Analytics for deployment metrics

### Regular Maintenance

- **Weekly:** Check for broken links
- **Monthly:** Review and update dependencies
- **Quarterly:** Performance audit with Lighthouse
- **As needed:** Update documentation content

## Best Practices

1. **Always test locally before pushing**
2. **Use preview deployments for major changes**
3. **Monitor build times and optimize as needed**
4. **Keep dependencies up to date**
5. **Regularly check for broken links**
6. **Monitor performance metrics**
7. **Set up alerts for deployment failures**

## Support

For deployment issues:
- Check GitHub Actions logs
- Review platform-specific logs (Netlify/Vercel)
- Consult Docusaurus documentation
- Open an issue in the repository

## Additional Resources

- [Docusaurus Deployment](https://docusaurus.io/docs/deployment)
- [GitHub Pages Documentation](https://docs.github.com/en/pages)
- [Netlify Documentation](https://docs.netlify.com)
- [Vercel Documentation](https://vercel.com/docs)
- [Docker Documentation](https://docs.docker.com)
