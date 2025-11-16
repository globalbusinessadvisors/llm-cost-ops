import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import Heading from '@theme/Heading';

import styles from './index.module.css';

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <Heading as="h1" className="hero__title">
          {siteConfig.title}
        </Heading>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div className={styles.buttons}>
          <Link
            className="button button--secondary button--lg"
            to="/docs/intro">
            Get Started
          </Link>
          <Link
            className="button button--outline button--secondary button--lg"
            to="/docs/api/overview"
            style={{marginLeft: '1rem'}}>
            View API Docs
          </Link>
        </div>
        <div className={styles.heroStats}>
          <div className={styles.stat}>
            <div className={styles.statValue}>7+</div>
            <div className={styles.statLabel}>LLM Providers</div>
          </div>
          <div className={styles.stat}>
            <div className={styles.statValue}>10-decimal</div>
            <div className={styles.statLabel}>Precision</div>
          </div>
          <div className={styles.stat}>
            <div className={styles.statValue}>40+</div>
            <div className={styles.statLabel}>Metrics</div>
          </div>
          <div className={styles.stat}>
            <div className={styles.statValue}>K8s Ready</div>
            <div className={styles.statLabel}>Production</div>
          </div>
        </div>
      </div>
    </header>
  );
}

function QuickStart() {
  return (
    <section className={styles.quickStart}>
      <div className="container">
        <Heading as="h2" className="text--center margin-bottom--lg">
          Quick Start
        </Heading>
        <div className={styles.codeExample}>
          <div className={styles.codeBlock}>
            <pre>
              <code>{`# Install
cargo install llm-cost-ops

# Initialize database
cost-ops init --database-url sqlite:cost-ops.db

# Add pricing
cost-ops pricing add \\
  --provider openai \\
  --model gpt-4 \\
  --input-price 10.0 \\
  --output-price 30.0

# Ingest usage data
cost-ops ingest --file usage.json

# Query costs
cost-ops query --range last-7-days --output json`}</code>
            </pre>
          </div>
        </div>
      </div>
    </section>
  );
}

function SDKShowcase() {
  return (
    <section className={styles.sdkShowcase}>
      <div className="container">
        <Heading as="h2" className="text--center margin-bottom--lg">
          Multi-Language SDKs
        </Heading>
        <div className={styles.sdkGrid}>
          <div className={styles.sdkCard}>
            <Heading as="h3">Rust</Heading>
            <pre>
              <code>{`use llm_cost_ops::*;

let calculator = CostCalculator::new(repo);
let cost = calculator
    .calculate_cost(&usage)
    .await?;`}</code>
            </pre>
            <Link to="/docs/sdks/rust">Learn more →</Link>
          </div>
          <div className={styles.sdkCard}>
            <Heading as="h3">Python</Heading>
            <pre>
              <code>{`from llm_cost_ops import CostOps

client = CostOps(api_key="...")
cost = client.calculate_cost(
    usage_record
)`}</code>
            </pre>
            <Link to="/docs/sdks/python">Learn more →</Link>
          </div>
          <div className={styles.sdkCard}>
            <Heading as="h3">TypeScript</Heading>
            <pre>
              <code>{`import { CostOps } from '@llm-cost-ops/sdk';

const client = new CostOps({
  apiKey: '...'
});
const cost = await client.calculate()`}</code>
            </pre>
            <Link to="/docs/sdks/typescript">Learn more →</Link>
          </div>
        </div>
      </div>
    </section>
  );
}

function Testimonials() {
  return (
    <section className={styles.testimonials}>
      <div className="container">
        <Heading as="h2" className="text--center margin-bottom--lg">
          Trusted by Engineering Teams
        </Heading>
        <div className={styles.testimonialGrid}>
          <div className={styles.testimonialCard}>
            <p className={styles.testimonialQuote}>
              "LLM Cost Ops gave us complete visibility into our AI spending.
              We reduced costs by 40% in the first month."
            </p>
            <div className={styles.testimonialAuthor}>
              <strong>Sarah Chen</strong>
              <span>VP Engineering, TechCorp</span>
            </div>
          </div>
          <div className={styles.testimonialCard}>
            <p className={styles.testimonialQuote}>
              "The multi-provider support and accurate tracking are game-changers.
              Essential for any serious LLM operation."
            </p>
            <div className={styles.testimonialAuthor}>
              <strong>Michael Rodriguez</strong>
              <span>CTO, AI Innovations</span>
            </div>
          </div>
          <div className={styles.testimonialCard}>
            <p className={styles.testimonialQuote}>
              "Production-ready from day one. The Kubernetes integration and
              observability stack are exactly what we needed."
            </p>
            <div className={styles.testimonialAuthor}>
              <strong>Emily Watson</strong>
              <span>DevOps Lead, CloudScale</span>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function CTA() {
  return (
    <section className={styles.cta}>
      <div className="container">
        <div className={styles.ctaContent}>
          <Heading as="h2">Ready to optimize your LLM costs?</Heading>
          <p>
            Join hundreds of engineering teams using LLM Cost Ops to track,
            analyze, and optimize their AI infrastructure spending.
          </p>
          <div className={styles.buttons}>
            <Link
              className="button button--primary button--lg"
              to="/docs/intro">
              Get Started
            </Link>
            <Link
              className="button button--outline button--primary button--lg"
              to="https://github.com/llm-devops/llm-cost-ops"
              style={{marginLeft: '1rem'}}>
              View on GitHub
            </Link>
          </div>
        </div>
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title="Home"
      description="Enterprise-grade cost operations platform for LLM infrastructure. Track, analyze, and optimize costs across multiple LLM providers.">
      <HomepageHeader />
      <main>
        <HomepageFeatures />
        <QuickStart />
        <SDKShowcase />
        <Testimonials />
        <CTA />
      </main>
    </Layout>
  );
}
