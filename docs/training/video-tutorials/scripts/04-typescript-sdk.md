# Video 04: TypeScript SDK Comprehensive Guide

## Metadata

- **Duration**: 22-25 minutes
- **Level**: Intermediate
- **Prerequisites**: Videos 01, 02
- **Target Audience**: TypeScript/JavaScript developers
- **Video ID**: LLMCO-V04-TYPESCRIPT
- **Version**: 1.0.0

## Learning Objectives

- Master TypeScript SDK features with full type safety
- Implement cost tracking in Node.js, Next.js, and browser environments
- Use decorators and middleware patterns
- Integrate with popular frameworks (Vercel AI SDK, LangChain.js)
- Handle streaming responses and real-time tracking
- Optimize bundle size for frontend applications
- Deploy serverless functions with cost tracking

## Equipment/Software Needed

- Screen recording software (1920x1080, 30fps)
- Node.js 18+, TypeScript 5+
- VS Code with TypeScript extensions
- Multiple project types: Node.js, Next.js, Express
- LLM Cost Ops instance running

## Scene Breakdown

### Scene 1: Opening & TypeScript Advantages
**Duration**: 0:00-1:30

**Narration**:
"Welcome to the TypeScript SDK guide! If you're building AI applications with TypeScript or JavaScript, this video covers everything you need. The SDK is fully typed, works in Node.js, browsers, and edge runtimes, and integrates seamlessly with modern frameworks. Let's dive into type-safe cost tracking!"

**On-Screen Text**:
- "TypeScript SDK Features:"
  - "Full type safety"
  - "Node.js, Browser, Edge support"
  - "Framework integrations"
  - "Streaming support"
  - "Decorator patterns"

---

### Scene 2: Type-Safe Configuration
**Duration**: 1:30-4:00

**Code/Demo**:
```typescript
import { CostTracker, TrackerConfig } from 'llm-cost-ops';

// Full type safety with IntelliSense
const config: TrackerConfig = {
  apiKey: process.env.LCOPS_API_KEY!,
  endpoint: process.env.LCOPS_ENDPOINT,

  // Type-checked options
  maxRetries: 3,
  retryDelay: 1000,
  bufferSize: 100,
  bufferTimeout: 5000,

  // Typed default tags
  defaultTags: {
    application: 'my-app',
    environment: process.env.NODE_ENV as 'development' | 'production',
    version: '1.0.0'
  },

  // Type-safe error handler
  onError: (error: Error, context: RequestContext) => {
    console.error('Tracking failed:', error.message);
    // Error handling logic
  }
};

const tracker = new CostTracker(config);
```

**Highlight**: "Full IntelliSense support • Type-safe configuration • Compile-time error checking"

---

### Scene 3: Decorator Pattern
**Duration**: 4:00-7:00

**Code/Demo**:
```typescript
import { track, TrackingOptions } from 'llm-cost-ops';

class ChatService {
  private openai = new OpenAI();

  // Decorator for automatic tracking
  @track({ tags: { feature: 'chat' } })
  async generateResponse(userId: string, message: string): Promise<string> {
    const response = await this.openai.chat.completions.create({
      model: 'gpt-4',
      messages: [
        { role: 'user', content: message }
      ]
    });

    return response.choices[0].message.content;
  }

  // Conditional tracking with parameters
  @track((userId: string) => ({
    tags: { userId, feature: 'summarization' },
    enabled: userId !== 'test-user'  // Skip tracking for test users
  }))
  async summarize(userId: string, text: string): Promise<string> {
    // Implementation
  }
}

// Usage - tracking happens automatically
const service = new ChatService();
const response = await service.generateResponse('user123', 'Hello!');
```

**Highlight**: "Decorators for clean code • Conditional tracking • Method-level granularity"

---

### Scene 4: Express.js Middleware
**Duration**: 7:00-9:30

**Code/Demo**:
```typescript
import express from 'express';
import { createTrackingMiddleware } from 'llm-cost-ops/middleware';

const app = express();

// Global tracking middleware
app.use(createTrackingMiddleware({
  tracker,
  extractTags: (req) => ({
    userId: req.user?.id,
    endpoint: req.path,
    method: req.method
  })
}));

// Route with automatic tracking
app.post('/api/chat', async (req, res) => {
  const { message } = req.body;

  // This LLM call is automatically tracked
  const response = await openai.chat.completions.create({
    model: 'gpt-4',
    messages: [{ role: 'user', content: message }]
  });

  res.json({ response: response.choices[0].message.content });
});

// Tracking data is available in request context
app.get('/api/cost-summary', async (req, res) => {
  const costs = await req.trackingContext.getCosts();
  res.json(costs);
});
```

**Highlight**: "Express middleware • Automatic request context • Per-route tracking"

---

### Scene 5: Next.js Integration
**Duration**: 9:30-12:30

**Code/Demo**:
```typescript
// app/api/chat/route.ts
import { NextRequest, NextResponse } from 'next/server';
import { CostTracker } from 'llm-cost-ops';
import { OpenAI } from 'openai';

const tracker = new CostTracker({
  apiKey: process.env.LCOPS_API_KEY!
});

export async function POST(request: NextRequest) {
  const { message } = await request.json();

  // Track in API route
  const response = await tracker.track(
    openai.chat.completions.create({
      model: 'gpt-4',
      messages: [{ role: 'user', content: message }]
    }),
    {
      tags: {
        route: '/api/chat',
        userId: request.headers.get('x-user-id')
      }
    }
  );

  return NextResponse.json({
    message: response.choices[0].message.content
  });
}

// Edge runtime support
export const runtime = 'edge';
```

**Server Component:**
```typescript
// app/dashboard/page.tsx
import { CostTracker } from 'llm-cost-ops';

async function getDailyCosts() {
  const tracker = new CostTracker({
    apiKey: process.env.LCOPS_API_KEY!
  });

  return tracker.getCosts({
    startDate: new Date(Date.now() - 24 * 60 * 60 * 1000),
    endDate: new Date()
  });
}

export default async function DashboardPage() {
  const costs = await getDailyCosts();

  return (
    <div>
      <h1>Today's Costs: ${costs.total}</h1>
      {/* Render cost data */}
    </div>
  );
}
```

**Highlight**: "App Router support • Edge runtime compatible • Server Components integration"

---

### Scene 6: Streaming Response Tracking
**Duration**: 12:30-15:00

**Code/Demo**:
```typescript
import { CostTracker } from 'llm-cost-ops';
import OpenAI from 'openai';

const tracker = new CostTracker({ apiKey: process.env.LCOPS_API_KEY! });
const openai = new OpenAI();

async function streamCompletion(prompt: string) {
  // Create tracked stream
  const stream = await tracker.trackStream(
    openai.chat.completions.create({
      model: 'gpt-4',
      messages: [{ role: 'user', content: prompt }],
      stream: true
    }),
    {
      tags: { feature: 'streaming-chat' }
    }
  );

  // Consume stream
  let fullContent = '';
  for await (const chunk of stream) {
    const delta = chunk.choices[0]?.delta?.content || '';
    fullContent += delta;
    process.stdout.write(delta);
  }

  // Costs tracked automatically after stream completes
  console.log('\n✅ Stream complete and tracked');
  return fullContent;
}

// Streaming to client in Next.js
export async function POST(request: NextRequest) {
  const { message } = await request.json();

  const stream = await tracker.trackStream(
    openai.chat.completions.create({
      model: 'gpt-4',
      messages: [{ role: 'user', content: message }],
      stream: true
    }),
    { tags: { userId: request.headers.get('x-user-id') } }
  );

  // Stream to client
  return new Response(stream.toReadableStream(), {
    headers: { 'Content-Type': 'text/event-stream' }
  });
}
```

**Highlight**: "Stream tracking • Real-time cost calculation • Client streaming support"

---

### Scene 7: Vercel AI SDK Integration
**Duration**: 15:00-17:00

**Code/Demo**:
```typescript
import { streamText } from 'ai';
import { openai } from '@ai-sdk/openai';
import { createTrackedProvider } from 'llm-cost-ops/integrations/vercel-ai';

// Wrap Vercel AI SDK provider with tracking
const trackedOpenAI = createTrackedProvider(openai, tracker, {
  defaultTags: { provider: 'vercel-ai-sdk' }
});

// Use in API route
export async function POST(req: Request) {
  const { messages } = await req.json();

  const result = await streamText({
    model: trackedOpenAI('gpt-4'),  // Automatic tracking
    messages,
  });

  return result.toAIStreamResponse();
}

// React hook with tracking
'use client';
import { useChat } from 'ai/react';

export function Chat() {
  const { messages, input, handleInputChange, handleSubmit } = useChat({
    api: '/api/chat',  // Uses tracked endpoint
    onFinish: (message) => {
      // Access tracking data
      console.log('Message cost:', message.metadata?.cost);
    }
  });

  return (
    <div>
      {messages.map(m => (
        <div key={m.id}>
          {m.role}: {m.content}
          {m.metadata?.cost && <span>${m.metadata.cost}</span>}
        </div>
      ))}
      <form onSubmit={handleSubmit}>
        <input value={input} onChange={handleInputChange} />
      </form>
    </div>
  );
}
```

**Highlight**: "Vercel AI SDK integration • React hooks • Streaming with costs"

---

### Scene 8: Browser & Bundle Optimization
**Duration**: 17:00-19:00

**Code/Demo**:
```typescript
// Lightweight browser bundle
import { CostTrackerLite } from 'llm-cost-ops/lite';

// Smaller bundle for browser - only client features
const tracker = new CostTrackerLite({
  apiKey: process.env.NEXT_PUBLIC_LCOPS_KEY!,
  endpoint: '/api/track'  // Proxy through your backend
});

// Track client-side LLM calls
async function clientSideChat(message: string) {
  const response = await fetch('/api/openai', {
    method: 'POST',
    body: JSON.stringify({ message })
  });

  const data = await response.json();

  // Track on client side
  tracker.trackEvent({
    type: 'completion',
    tokens: data.usage.total_tokens,
    cost: data.cost,
    tags: {
      source: 'client',
      userId: getCurrentUserId()
    }
  });

  return data.message;
}
```

**Bundle size comparison:**
```typescript
// Full SDK: 45KB gzipped
import { CostTracker } from 'llm-cost-ops';

// Lite SDK: 8KB gzipped
import { CostTrackerLite } from 'llm-cost-ops/lite';

// Tree-shaking support - import only what you need
import { track } from 'llm-cost-ops/decorators';
import { createMiddleware } from 'llm-cost-ops/middleware';
```

**Highlight**: "Lite bundle for browsers • Tree-shaking support • 80% smaller for client"

---

### Scene 9: Error Handling & Type Safety
**Duration**: 19:00-21:00

**Code/Demo**:
```typescript
import { CostTracker, TrackingError, NetworkError } from 'llm-cost-ops';

const tracker = new CostTracker({
  apiKey: process.env.LCOPS_API_KEY!,

  // Type-safe error handlers
  onError: (error: Error, context: RequestContext) => {
    if (error instanceof NetworkError) {
      // Handle network errors
      logger.warn('Tracking network error', { error });
      queueForRetry(context);
    } else if (error instanceof TrackingError) {
      // Handle tracking-specific errors
      logger.error('Tracking failed', { error, context });
      sendToMonitoring(error);
    }
  }
});

// Type-safe tracking with error handling
async function safeTrack<T>(
  operation: Promise<T>,
  tags?: Record<string, string>
): Promise<{ result: T; tracked: boolean }> {
  try {
    const result = await tracker.track(operation, { tags });
    return { result, tracked: true };
  } catch (error) {
    if (error instanceof TrackingError) {
      // Tracking failed but operation succeeded
      return {
        result: await operation,  // Get result without tracking
        tracked: false
      };
    }
    throw error;  // Re-throw other errors
  }
}

// Usage with full type safety
const { result, tracked } = await safeTrack(
  openai.chat.completions.create({
    model: 'gpt-4',
    messages: [{ role: 'user', content: 'Hello' }]
  }),
  { userId: '123' }
);

console.log(`Response: ${result.choices[0].message.content}`);
console.log(`Tracking ${tracked ? 'succeeded' : 'failed'}`);
```

**Highlight**: "Type-safe error handling • Graceful degradation • Full TypeScript support"

---

### Scene 10: Advanced Patterns & Best Practices
**Duration**: 21:00-23:00

**Code/Demo**:
```typescript
// Singleton pattern for tracker
class TrackerSingleton {
  private static instance: CostTracker;

  static getInstance(): CostTracker {
    if (!TrackerSingleton.instance) {
      TrackerSingleton.instance = new CostTracker({
        apiKey: process.env.LCOPS_API_KEY!,
        // ... config
      });
    }
    return TrackerSingleton.instance;
  }
}

// Factory pattern for different environments
class TrackerFactory {
  static create(env: 'development' | 'production' | 'test'): CostTracker {
    const baseConfig = {
      apiKey: process.env.LCOPS_API_KEY!
    };

    switch (env) {
      case 'development':
        return new CostTracker({
          ...baseConfig,
          debug: true,
          sampling: 1.0
        });
      case 'production':
        return new CostTracker({
          ...baseConfig,
          sampling: 0.1,
          bufferSize: 1000
        });
      case 'test':
        return new CostTrackerMock();  // Mock for tests
    }
  }
}

// Dependency injection
import { Container } from 'inversify';

container.bind<CostTracker>('CostTracker').toDynamicValue(() => {
  return new CostTracker({
    apiKey: process.env.LCOPS_API_KEY!
  });
}).inSingletonScope();

// Use in services
class ChatService {
  constructor(
    @inject('CostTracker') private tracker: CostTracker
  ) {}

  async chat(message: string) {
    return this.tracker.track(
      // LLM call
    );
  }
}
```

**Testing utilities:**
```typescript
import { MockTracker, createTestTracker } from 'llm-cost-ops/testing';

describe('ChatService', () => {
  it('should track LLM costs', async () => {
    const mockTracker = createTestTracker();
    const service = new ChatService(mockTracker);

    await service.chat('Hello');

    expect(mockTracker.trackedRequests).toHaveLength(1);
    expect(mockTracker.totalCost).toBeGreaterThan(0);
  });
});
```

**Highlight**: "Design patterns • Testing utilities • Production-ready architecture"

---

### Scene 11: Recap & Resources
**Duration**: 23:00-24:00

**Narration**:
"That's the complete TypeScript SDK guide! We covered type-safe configuration, decorators, middleware, Next.js integration, streaming, Vercel AI SDK, bundle optimization, and production patterns. All code examples are on GitHub. Next up: building custom analytics dashboards. Thanks for watching!"

**On-Screen Text**:
- "Covered:"
  - "✅ Type-safe configuration"
  - "✅ Decorators & middleware"
  - "✅ Framework integrations"
  - "✅ Streaming support"
  - "✅ Production patterns"
- "Next: Video 05 - Analytics Dashboards"

---

## Post-Production Notes

### Chapter Markers
- 0:00 - Introduction
- 1:30 - Type-Safe Configuration
- 4:00 - Decorator Pattern
- 7:00 - Express Middleware
- 9:30 - Next.js Integration
- 12:30 - Streaming Tracking
- 15:00 - Vercel AI SDK
- 17:00 - Bundle Optimization
- 19:00 - Error Handling
- 21:00 - Advanced Patterns
- 23:00 - Recap

### Code Repository
- TypeScript examples repository
- Next.js starter template
- Express API template
- Testing examples

**Script Version**: 1.0.0
**Last Updated**: 2025-11-16
