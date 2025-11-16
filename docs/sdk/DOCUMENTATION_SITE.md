# Documentation Website Setup

This guide explains how to set up the unified documentation website for LLM-CostOps using Docusaurus.

## Overview

The documentation site provides:
- Unified documentation for all SDKs (Python, TypeScript, Go, Java)
- Interactive API reference
- Code examples and tutorials
- Search functionality
- Version management
- Mobile-friendly design

## Technology Stack

**Recommended:** Docusaurus v3
- React-based
- Fast and modern
- Excellent search (Algolia)
- Version management built-in
- MDX support (React components in Markdown)
- Dark mode support

**Alternatives:**
- VitePress (Vue-based, faster builds)
- MkDocs (Python, simpler)
- GitBook (commercial, easiest)

## Quick Start with Docusaurus

### 1. Install Docusaurus

```bash
cd /workspaces/llm-cost-ops
npx create-docusaurus@latest docs-site classic
cd docs-site
```

### 2. Directory Structure

```
docs-site/
├── docs/                    # Documentation files
│   ├── getting-started/
│   │   ├── quickstart.md
│   │   ├── authentication.md
│   │   └── installation.md
│   ├── guides/
│   │   ├── cost-analysis.md
│   │   ├── forecasting.md
│   │   └── budget-management.md
│   ├── api-reference/
│   │   ├── rest-api/
│   │   ├── python/
│   │   ├── typescript/
│   │   ├── go/
│   │   └── java/
│   ├── examples/
│   │   ├── curl/
│   │   ├── python/
│   │   ├── typescript/
│   │   ├── go/
│   │   └── java/
│   ├── frameworks/
│   │   ├── fastapi.md
│   │   ├── django.md
│   │   ├── react.md
│   │   └── nextjs.md
│   ├── troubleshooting.md
│   └── faq.md
├── blog/                    # Blog posts (optional)
├── src/
│   ├── components/          # Custom React components
│   ├── css/                 # Custom styles
│   └── pages/               # Custom pages
├── static/                  # Static files (images, etc.)
├── docusaurus.config.js     # Main configuration
├── sidebars.js              # Sidebar structure
└── package.json
```

### 3. Configuration

**docusaurus.config.js:**
```javascript
module.exports = {
  title: 'LLM-CostOps Documentation',
  tagline: 'Enterprise-grade cost operations for LLM infrastructure',
  url: 'https://docs.llm-cost-ops.dev',
  baseUrl: '/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',
  favicon: 'img/favicon.ico',
  organizationName: 'llm-devops',
  projectName: 'llm-cost-ops',

  presets: [
    [
      '@docusaurus/preset-classic',
      {
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          editUrl: 'https://github.com/llm-devops/llm-cost-ops/edit/main/docs/',
          showLastUpdateAuthor: true,
          showLastUpdateTime: true,
        },
        blog: {
          showReadingTime: true,
          editUrl: 'https://github.com/llm-devops/llm-cost-ops/edit/main/blog/',
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      },
    ],
  ],

  themeConfig: {
    navbar: {
      title: 'LLM-CostOps',
      logo: {
        alt: 'LLM-CostOps Logo',
        src: 'img/logo.svg',
      },
      items: [
        {
          type: 'doc',
          docId: 'getting-started/quickstart',
          position: 'left',
          label: 'Docs',
        },
        {
          type: 'doc',
          docId: 'api-reference/rest-api/README',
          position: 'left',
          label: 'API Reference',
        },
        {
          to: '/examples',
          label: 'Examples',
          position: 'left',
        },
        {
          href: 'https://github.com/llm-devops/llm-cost-ops',
          label: 'GitHub',
          position: 'right',
        },
        {
          type: 'search',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {
              label: 'Quickstart',
              to: '/docs/getting-started/quickstart',
            },
            {
              label: 'API Reference',
              to: '/docs/api-reference/rest-api',
            },
            {
              label: 'Examples',
              to: '/examples',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'Discord',
              href: 'https://discord.gg/llm-cost-ops',
            },
            {
              label: 'GitHub Discussions',
              href: 'https://github.com/llm-devops/llm-cost-ops/discussions',
            },
            {
              label: 'Twitter',
              href: 'https://twitter.com/llmcostops',
            },
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'Blog',
              to: '/blog',
            },
            {
              label: 'GitHub',
              href: 'https://github.com/llm-devops/llm-cost-ops',
            },
            {
              label: 'Status',
              href: 'https://status.llm-cost-ops.dev',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} LLM-CostOps. Built with Docusaurus.`,
    },
    prism: {
      theme: require('prism-react-renderer/themes/github'),
      darkTheme: require('prism-react-renderer/themes/dracula'),
      additionalLanguages: ['rust', 'python', 'typescript', 'go', 'java', 'bash'],
    },
    algolia: {
      appId: 'YOUR_APP_ID',
      apiKey: 'YOUR_SEARCH_API_KEY',
      indexName: 'llm-cost-ops',
    },
  },
};
```

**sidebars.js:**
```javascript
module.exports = {
  docs: [
    {
      type: 'category',
      label: 'Getting Started',
      items: [
        'getting-started/quickstart',
        'getting-started/authentication',
        'getting-started/installation',
      ],
    },
    {
      type: 'category',
      label: 'Guides',
      items: [
        'guides/cost-analysis',
        'guides/forecasting',
        'guides/budget-management',
        'guides/export-reports',
        'guides/anomaly-detection',
      ],
    },
    {
      type: 'category',
      label: 'API Reference',
      items: [
        {
          type: 'category',
          label: 'REST API',
          items: [
            'api-reference/rest-api/README',
            'api-reference/rest-api/authentication',
            'api-reference/rest-api/usage',
            'api-reference/rest-api/costs',
            'api-reference/rest-api/pricing',
            'api-reference/rest-api/analytics',
          ],
        },
        {
          type: 'category',
          label: 'Python SDK',
          items: [
            'api-reference/python/README',
            'api-reference/python/client',
            'api-reference/python/usage',
            'api-reference/python/costs',
          ],
        },
        {
          type: 'category',
          label: 'TypeScript SDK',
          items: [
            'api-reference/typescript/README',
            'api-reference/typescript/client',
            'api-reference/typescript/usage',
            'api-reference/typescript/costs',
          ],
        },
      ],
    },
    {
      type: 'category',
      label: 'Examples',
      items: [
        'examples/curl/README',
        'examples/python/README',
        'examples/typescript/README',
        'examples/go/README',
        'examples/java/README',
      ],
    },
    {
      type: 'category',
      label: 'Framework Integration',
      items: [
        'frameworks/fastapi',
        'frameworks/django',
        'frameworks/flask',
        'frameworks/react',
        'frameworks/nextjs',
        'frameworks/spring-boot',
      ],
    },
    'troubleshooting',
    'faq',
  ],
};
```

### 4. Custom Components

**Interactive API Explorer Component:**

```jsx
// src/components/ApiExplorer.js
import React, { useState } from 'react';

export default function ApiExplorer() {
  const [apiKey, setApiKey] = useState('');
  const [response, setResponse] = useState(null);

  const fetchCosts = async () => {
    const res = await fetch('https://api.llm-cost-ops.dev/api/v1/costs', {
      headers: {
        'Authorization': `Bearer ${apiKey}`
      }
    });
    const data = await res.json();
    setResponse(JSON.stringify(data, null, 2));
  };

  return (
    <div className="api-explorer">
      <h3>Try it out</h3>
      <input
        type="text"
        placeholder="API Key"
        value={apiKey}
        onChange={(e) => setApiKey(e.target.value)}
      />
      <button onClick={fetchCosts}>Get Costs</button>
      {response && (
        <pre><code>{response}</code></pre>
      )}
    </div>
  );
}
```

**Code Switcher Component:**

```jsx
// src/components/CodeSwitcher.js
import React, { useState } from 'react';
import CodeBlock from '@theme/CodeBlock';

export default function CodeSwitcher({ examples }) {
  const [language, setLanguage] = useState(Object.keys(examples)[0]);

  return (
    <div>
      <div className="language-tabs">
        {Object.keys(examples).map(lang => (
          <button
            key={lang}
            onClick={() => setLanguage(lang)}
            className={language === lang ? 'active' : ''}
          >
            {lang}
          </button>
        ))}
      </div>
      <CodeBlock language={language}>
        {examples[language]}
      </CodeBlock>
    </div>
  );
}
```

**Usage in Markdown:**

```mdx
import CodeSwitcher from '@site/src/components/CodeSwitcher';

# Submit Usage

<CodeSwitcher
  examples={{
    'curl': `curl -X POST https://api.llm-cost-ops.dev/api/v1/usage \\
  -H "Authorization: Bearer $API_KEY" \\
  -d '{"organization_id": "org-123", ...}'`,
    'python': `client.usage.submit(
  organization_id="org-123",
  provider="openai",
  model_id="gpt-4",
  ...
)`,
    'typescript': `await client.usage.submit({
  organizationId: 'org-123',
  provider: 'openai',
  modelId: 'gpt-4',
  ...
})`,
  }}
/>
```

### 5. Build and Deploy

**Development:**
```bash
npm start
# Opens http://localhost:3000
```

**Build:**
```bash
npm run build
# Output in build/
```

**Deploy:**

**Netlify:**
```bash
npm install -g netlify-cli
netlify deploy --prod --dir=build
```

**Vercel:**
```bash
npm install -g vercel
vercel --prod
```

**GitHub Pages:**
```yaml
# .github/workflows/deploy-docs.yml
name: Deploy Docs
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions/setup-node@v2
        with:
          node-version: 18
      - run: cd docs-site && npm install
      - run: cd docs-site && npm run build
      - uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./docs-site/build
```

**Custom Server:**
```bash
# Build
npm run build

# Serve with nginx
server {
    listen 80;
    server_name docs.llm-cost-ops.dev;
    root /var/www/docs/build;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

## OpenAPI/Swagger Integration

Generate interactive API documentation from OpenAPI spec:

### 1. Create OpenAPI Spec

```yaml
# openapi.yaml
openapi: 3.0.0
info:
  title: LLM-CostOps API
  version: 1.0.0
  description: Enterprise-grade cost operations for LLM infrastructure

servers:
  - url: https://api.llm-cost-ops.dev
    description: Production API

paths:
  /api/v1/usage:
    post:
      summary: Submit usage record
      security:
        - bearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/UsageRequest'
      responses:
        '201':
          description: Usage submitted successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/UsageResponse'

components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
  schemas:
    UsageRequest:
      type: object
      required:
        - organization_id
        - provider
        - model_id
        - input_tokens
        - output_tokens
        - total_tokens
      properties:
        organization_id:
          type: string
        provider:
          type: string
          enum: [openai, anthropic, google, azure, aws]
        model_id:
          type: string
        input_tokens:
          type: integer
        output_tokens:
          type: integer
        total_tokens:
          type: integer
```

### 2. Integrate with Docusaurus

```bash
npm install docusaurus-plugin-openapi-docs
npm install docusaurus-theme-openapi-docs
```

```javascript
// docusaurus.config.js
module.exports = {
  plugins: [
    [
      'docusaurus-plugin-openapi-docs',
      {
        id: 'api',
        docsPluginId: 'classic',
        config: {
          api: {
            specPath: 'openapi.yaml',
            outputDir: 'docs/api-reference/rest-api',
            sidebarOptions: {
              groupPathsBy: 'tag',
            },
          },
        },
      },
    ],
  ],
  themes: ['docusaurus-theme-openapi-docs'],
};
```

## Search Setup (Algolia)

### 1. Configure Algolia

```javascript
// docusaurus.config.js
module.exports = {
  themeConfig: {
    algolia: {
      appId: 'YOUR_APP_ID',
      apiKey: 'YOUR_SEARCH_API_KEY',
      indexName: 'llm-cost-ops',
      contextualSearch: true,
      searchParameters: {},
    },
  },
};
```

### 2. Create Crawler Config

```json
{
  "index_name": "llm-cost-ops",
  "start_urls": ["https://docs.llm-cost-ops.dev/"],
  "selectors": {
    "lvl0": "header h1",
    "lvl1": "article h2",
    "lvl2": "article h3",
    "lvl3": "article h4",
    "text": "article p, article li"
  }
}
```

## Analytics

Add Google Analytics or Plausible:

```javascript
// docusaurus.config.js
module.exports = {
  presets: [
    [
      '@docusaurus/preset-classic',
      {
        gtag: {
          trackingID: 'G-XXXXXXXXXX',
          anonymizeIP: true,
        },
      },
    ],
  ],
};
```

Or Plausible (privacy-friendly):

```javascript
module.exports = {
  scripts: [
    {
      src: 'https://plausible.io/js/script.js',
      defer: true,
      'data-domain': 'docs.llm-cost-ops.dev',
    },
  ],
};
```

## Versioning

Support multiple SDK versions:

```bash
npm run docusaurus docs:version 1.0.0
```

This creates:
- `versioned_docs/version-1.0.0/`
- `versioned_sidebars/version-1.0.0-sidebars.json`
- Updates `versions.json`

## Internationalization (i18n)

```javascript
// docusaurus.config.js
module.exports = {
  i18n: {
    defaultLocale: 'en',
    locales: ['en', 'fr', 'de', 'ja'],
  },
};
```

## Best Practices

1. **Keep docs in sync with code**: Auto-generate API docs
2. **Test all code examples**: Ensure they work
3. **Use MDX components**: Make docs interactive
4. **Enable search**: Essential for large docs
5. **Mobile-friendly**: Test on mobile devices
6. **Fast loading**: Optimize images, lazy load
7. **Analytics**: Track popular pages
8. **Feedback widget**: Let users report issues
9. **Version docs**: Support multiple versions
10. **CI/CD**: Auto-deploy on merge to main

## Next Steps

1. Copy documentation from `/workspaces/llm-cost-ops/docs/sdk/` to Docusaurus
2. Set up Docusaurus configuration
3. Add custom components
4. Configure search (Algolia)
5. Set up CI/CD pipeline
6. Deploy to production
7. Add analytics
8. Enable feedback mechanism

## Resources

- Docusaurus: https://docusaurus.io/
- OpenAPI plugin: https://github.com/PaloAltoNetworks/docusaurus-openapi-docs
- Algolia search: https://docsearch.algolia.com/
- Deployment guides: https://docusaurus.io/docs/deployment
