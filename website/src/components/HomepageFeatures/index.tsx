import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  icon: string;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Multi-Provider Support',
    icon: '🌐',
    description: (
      <>
        Track costs across OpenAI, Anthropic, Google Vertex AI, Azure OpenAI,
        AWS Bedrock, Cohere, and Mistral with a unified API.
      </>
    ),
  },
  {
    title: 'High Precision Calculations',
    icon: '🎯',
    description: (
      <>
        10-decimal precision using rust_decimal ensures accurate financial
        calculations for enterprise-grade cost tracking.
      </>
    ),
  },
  {
    title: 'Production Ready',
    icon: '🚀',
    description: (
      <>
        Kubernetes-ready with Helm charts, comprehensive observability,
        and enterprise security features out of the box.
      </>
    ),
  },
  {
    title: 'Advanced Analytics',
    icon: '📊',
    description: (
      <>
        Time-series forecasting, anomaly detection, and budget alerts help
        you stay ahead of cost overruns.
      </>
    ),
  },
  {
    title: 'Export & Reporting',
    icon: '📄',
    description: (
      <>
        Automated scheduled reports in CSV, JSON, or Excel with email delivery
        and webhook integration.
      </>
    ),
  },
  {
    title: 'Full Observability',
    icon: '🔍',
    description: (
      <>
        40+ Prometheus metrics, distributed tracing, structured logging,
        and comprehensive health checks.
      </>
    ),
  },
];

function Feature({title, icon, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <div className={styles.featureIcon}>{icon}</div>
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
