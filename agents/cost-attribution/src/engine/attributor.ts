import Decimal from 'decimal.js';
import { CostRecord, Currency } from './calculator';

/**
 * Attribution base interface
 */
export interface Attribution {
  scope: string;
  scopeType: 'execution' | 'agent' | 'workflow' | 'tenant';
  totalCost: string;
  currency: Currency;
  recordCount: number;
  startTime: Date;
  endTime: Date;
}

/**
 * Execution-level attribution
 */
export interface ExecutionAttribution extends Attribution {
  scopeType: 'execution';
  executionId: string;
  agentId: string;
  workflowId?: string;
  tenantId?: string;

  totalInputTokens: number;
  totalOutputTokens: number;
  totalCachedInputTokens: number;

  providerBreakdown: ProviderBreakdown[];
}

/**
 * Agent-level attribution (aggregated across executions)
 */
export interface AgentAttribution extends Attribution {
  scopeType: 'agent';
  agentId: string;

  executionCount: number;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalCachedInputTokens: number;

  providerBreakdown: ProviderBreakdown[];
  modelBreakdown: ModelBreakdown[];
}

/**
 * Workflow-level attribution (aggregated across agents)
 */
export interface WorkflowAttribution extends Attribution {
  scopeType: 'workflow';
  workflowId: string;

  agentCount: number;
  executionCount: number;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalCachedInputTokens: number;

  agentBreakdown: AgentBreakdown[];
  providerBreakdown: ProviderBreakdown[];
}

/**
 * Tenant-level attribution (multi-tenant cost allocation)
 */
export interface TenantAttribution extends Attribution {
  scopeType: 'tenant';
  tenantId: string;

  workflowCount: number;
  agentCount: number;
  executionCount: number;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalCachedInputTokens: number;

  workflowBreakdown: WorkflowBreakdown[];
  agentBreakdown: AgentBreakdown[];
  providerBreakdown: ProviderBreakdown[];
}

/**
 * Provider cost breakdown
 */
export interface ProviderBreakdown {
  provider: string;
  cost: string;
  inputTokens: number;
  outputTokens: number;
  cachedInputTokens: number;
  recordCount: number;
}

/**
 * Model cost breakdown
 */
export interface ModelBreakdown {
  model: string;
  provider: string;
  cost: string;
  inputTokens: number;
  outputTokens: number;
  cachedInputTokens: number;
  recordCount: number;
}

/**
 * Agent cost breakdown
 */
export interface AgentBreakdown {
  agentId: string;
  cost: string;
  executionCount: number;
  inputTokens: number;
  outputTokens: number;
  cachedInputTokens: number;
}

/**
 * Workflow cost breakdown
 */
export interface WorkflowBreakdown {
  workflowId: string;
  cost: string;
  agentCount: number;
  executionCount: number;
  inputTokens: number;
  outputTokens: number;
  cachedInputTokens: number;
}

/**
 * Attribution summary across all scopes
 */
export interface AttributionSummary {
  totalCost: string;
  currency: Currency;
  recordCount: number;
  startTime: Date;
  endTime: Date;

  executionCount: number;
  agentCount: number;
  workflowCount: number;
  tenantCount: number;

  providerBreakdown: ProviderBreakdown[];
  modelBreakdown: ModelBreakdown[];

  topAgents: AgentBreakdown[];
  topWorkflows: WorkflowBreakdown[];
}

/**
 * CostAttributor - Pure, deterministic cost attribution engine
 *
 * Attributes costs to different scopes (execution, agent, workflow, tenant)
 * using stateless aggregation functions.
 */
export class CostAttributor {
  /**
   * Attribute costs by individual execution
   */
  attributeByExecution(costs: CostRecord[]): ExecutionAttribution[] {
    const executionMap = new Map<string, CostRecord[]>();

    // Group by execution
    for (const cost of costs) {
      const existing = executionMap.get(cost.executionId) || [];
      existing.push(cost);
      executionMap.set(cost.executionId, existing);
    }

    // Aggregate each execution
    const attributions: ExecutionAttribution[] = [];
    for (const [executionId, records] of executionMap) {
      attributions.push(this.aggregateExecution(executionId, records));
    }

    return attributions;
  }

  /**
   * Attribute costs by agent (across all executions)
   */
  attributeByAgent(costs: CostRecord[]): AgentAttribution[] {
    const agentMap = new Map<string, CostRecord[]>();

    // Group by agent
    for (const cost of costs) {
      const existing = agentMap.get(cost.agentId) || [];
      existing.push(cost);
      agentMap.set(cost.agentId, existing);
    }

    // Aggregate each agent
    const attributions: AgentAttribution[] = [];
    for (const [agentId, records] of agentMap) {
      attributions.push(this.aggregateAgent(agentId, records));
    }

    return attributions;
  }

  /**
   * Attribute costs by workflow (across all agents)
   */
  attributeByWorkflow(costs: CostRecord[]): WorkflowAttribution[] {
    // Filter records with workflowId
    const workflowCosts = costs.filter(c => c.workflowId);
    const workflowMap = new Map<string, CostRecord[]>();

    // Group by workflow
    for (const cost of workflowCosts) {
      const workflowId = cost.workflowId!;
      const existing = workflowMap.get(workflowId) || [];
      existing.push(cost);
      workflowMap.set(workflowId, existing);
    }

    // Aggregate each workflow
    const attributions: WorkflowAttribution[] = [];
    for (const [workflowId, records] of workflowMap) {
      attributions.push(this.aggregateWorkflow(workflowId, records));
    }

    return attributions;
  }

  /**
   * Attribute costs by tenant (multi-tenant cost allocation)
   */
  attributeByTenant(costs: CostRecord[]): TenantAttribution[] {
    // Filter records with tenantId
    const tenantCosts = costs.filter(c => c.tenantId);
    const tenantMap = new Map<string, CostRecord[]>();

    // Group by tenant
    for (const cost of tenantCosts) {
      const tenantId = cost.tenantId!;
      const existing = tenantMap.get(tenantId) || [];
      existing.push(cost);
      tenantMap.set(tenantId, existing);
    }

    // Aggregate each tenant
    const attributions: TenantAttribution[] = [];
    for (const [tenantId, records] of tenantMap) {
      attributions.push(this.aggregateTenant(tenantId, records));
    }

    return attributions;
  }

  /**
   * Generate comprehensive summary across all attributions
   */
  generateSummary(attributions: Attribution[]): AttributionSummary {
    if (attributions.length === 0) {
      throw new Error('Cannot generate summary from empty attributions');
    }

    const currency = attributions[0].currency;
    let totalCost = new Decimal(0);
    let minTime = attributions[0].startTime;
    let maxTime = attributions[0].endTime;

    const executionIds = new Set<string>();
    const agentIds = new Set<string>();
    const workflowIds = new Set<string>();
    const tenantIds = new Set<string>();

    let totalRecords = 0;

    // Collect all cost records from attributions
    const allCosts: CostRecord[] = [];

    for (const attr of attributions) {
      totalCost = totalCost.plus(new Decimal(attr.totalCost));
      totalRecords += attr.recordCount;

      if (attr.startTime < minTime) minTime = attr.startTime;
      if (attr.endTime > maxTime) maxTime = attr.endTime;

      // Track unique IDs
      if (attr.scopeType === 'execution') {
        const exec = attr as ExecutionAttribution;
        executionIds.add(exec.executionId);
        agentIds.add(exec.agentId);
        if (exec.workflowId) workflowIds.add(exec.workflowId);
        if (exec.tenantId) tenantIds.add(exec.tenantId);
      } else if (attr.scopeType === 'agent') {
        const agent = attr as AgentAttribution;
        agentIds.add(agent.agentId);
      } else if (attr.scopeType === 'workflow') {
        const workflow = attr as WorkflowAttribution;
        workflowIds.add(workflow.workflowId);
      } else if (attr.scopeType === 'tenant') {
        const tenant = attr as TenantAttribution;
        tenantIds.add(tenant.tenantId);
      }
    }

    // Build summary breakdowns
    const providerMap = new Map<string, ProviderBreakdown>();
    const modelMap = new Map<string, ModelBreakdown>();
    const agentMap = new Map<string, AgentBreakdown>();
    const workflowMap = new Map<string, WorkflowBreakdown>();

    for (const attr of attributions) {
      // Aggregate provider breakdowns
      if ('providerBreakdown' in attr) {
        for (const pb of (attr as any).providerBreakdown) {
          const existing = providerMap.get(pb.provider) || {
            provider: pb.provider,
            cost: '0',
            inputTokens: 0,
            outputTokens: 0,
            cachedInputTokens: 0,
            recordCount: 0,
          };

          existing.cost = new Decimal(existing.cost).plus(new Decimal(pb.cost)).toFixed(10);
          existing.inputTokens += pb.inputTokens;
          existing.outputTokens += pb.outputTokens;
          existing.cachedInputTokens += pb.cachedInputTokens;
          existing.recordCount += pb.recordCount;

          providerMap.set(pb.provider, existing);
        }
      }

      // Aggregate model breakdowns
      if ('modelBreakdown' in attr) {
        for (const mb of (attr as any).modelBreakdown) {
          const key = `${mb.provider}:${mb.model}`;
          const existing = modelMap.get(key) || {
            model: mb.model,
            provider: mb.provider,
            cost: '0',
            inputTokens: 0,
            outputTokens: 0,
            cachedInputTokens: 0,
            recordCount: 0,
          };

          existing.cost = new Decimal(existing.cost).plus(new Decimal(mb.cost)).toFixed(10);
          existing.inputTokens += mb.inputTokens;
          existing.outputTokens += mb.outputTokens;
          existing.cachedInputTokens += mb.cachedInputTokens;
          existing.recordCount += mb.recordCount;

          modelMap.set(key, existing);
        }
      }

      // Aggregate agent breakdowns
      if ('agentBreakdown' in attr) {
        for (const ab of (attr as any).agentBreakdown) {
          const existing = agentMap.get(ab.agentId) || {
            agentId: ab.agentId,
            cost: '0',
            executionCount: 0,
            inputTokens: 0,
            outputTokens: 0,
            cachedInputTokens: 0,
          };

          existing.cost = new Decimal(existing.cost).plus(new Decimal(ab.cost)).toFixed(10);
          existing.executionCount += ab.executionCount;
          existing.inputTokens += ab.inputTokens;
          existing.outputTokens += ab.outputTokens;
          existing.cachedInputTokens += ab.cachedInputTokens;

          agentMap.set(ab.agentId, existing);
        }
      }

      // Aggregate workflow breakdowns
      if ('workflowBreakdown' in attr) {
        for (const wb of (attr as any).workflowBreakdown) {
          const existing = workflowMap.get(wb.workflowId) || {
            workflowId: wb.workflowId,
            cost: '0',
            agentCount: 0,
            executionCount: 0,
            inputTokens: 0,
            outputTokens: 0,
            cachedInputTokens: 0,
          };

          existing.cost = new Decimal(existing.cost).plus(new Decimal(wb.cost)).toFixed(10);
          existing.agentCount = Math.max(existing.agentCount, wb.agentCount);
          existing.executionCount += wb.executionCount;
          existing.inputTokens += wb.inputTokens;
          existing.outputTokens += wb.outputTokens;
          existing.cachedInputTokens += wb.cachedInputTokens;

          workflowMap.set(wb.workflowId, existing);
        }
      }
    }

    // Sort and get top agents
    const topAgents = Array.from(agentMap.values())
      .sort((a, b) => new Decimal(b.cost).minus(new Decimal(a.cost)).toNumber())
      .slice(0, 10);

    // Sort and get top workflows
    const topWorkflows = Array.from(workflowMap.values())
      .sort((a, b) => new Decimal(b.cost).minus(new Decimal(a.cost)).toNumber())
      .slice(0, 10);

    return {
      totalCost: totalCost.toFixed(10),
      currency,
      recordCount: totalRecords,
      startTime: minTime,
      endTime: maxTime,

      executionCount: executionIds.size,
      agentCount: agentIds.size,
      workflowCount: workflowIds.size,
      tenantCount: tenantIds.size,

      providerBreakdown: Array.from(providerMap.values()),
      modelBreakdown: Array.from(modelMap.values()),

      topAgents,
      topWorkflows,
    };
  }

  /**
   * Aggregate a single execution
   */
  private aggregateExecution(executionId: string, records: CostRecord[]): ExecutionAttribution {
    const currency = records[0].currency;
    let totalCost = new Decimal(0);
    let totalInputTokens = 0;
    let totalOutputTokens = 0;
    let totalCachedInputTokens = 0;
    let minTime = records[0].timestamp;
    let maxTime = records[0].timestamp;

    const providerMap = new Map<string, ProviderBreakdown>();

    for (const record of records) {
      totalCost = totalCost.plus(new Decimal(record.totalCost));
      totalInputTokens += record.inputTokens;
      totalOutputTokens += record.outputTokens;
      totalCachedInputTokens += record.cachedInputTokens;

      if (record.timestamp < minTime) minTime = record.timestamp;
      if (record.timestamp > maxTime) maxTime = record.timestamp;

      // Provider breakdown
      const existing = providerMap.get(record.provider) || {
        provider: record.provider,
        cost: '0',
        inputTokens: 0,
        outputTokens: 0,
        cachedInputTokens: 0,
        recordCount: 0,
      };

      existing.cost = new Decimal(existing.cost).plus(new Decimal(record.totalCost)).toFixed(10);
      existing.inputTokens += record.inputTokens;
      existing.outputTokens += record.outputTokens;
      existing.cachedInputTokens += record.cachedInputTokens;
      existing.recordCount++;

      providerMap.set(record.provider, existing);
    }

    return {
      scope: executionId,
      scopeType: 'execution',
      executionId,
      agentId: records[0].agentId,
      workflowId: records[0].workflowId,
      tenantId: records[0].tenantId,

      totalCost: totalCost.toFixed(10),
      currency,
      recordCount: records.length,
      startTime: minTime,
      endTime: maxTime,

      totalInputTokens,
      totalOutputTokens,
      totalCachedInputTokens,

      providerBreakdown: Array.from(providerMap.values()),
    };
  }

  /**
   * Aggregate an agent across executions
   */
  private aggregateAgent(agentId: string, records: CostRecord[]): AgentAttribution {
    const currency = records[0].currency;
    let totalCost = new Decimal(0);
    let totalInputTokens = 0;
    let totalOutputTokens = 0;
    let totalCachedInputTokens = 0;
    let minTime = records[0].timestamp;
    let maxTime = records[0].timestamp;

    const executionIds = new Set<string>();
    const providerMap = new Map<string, ProviderBreakdown>();
    const modelMap = new Map<string, ModelBreakdown>();

    for (const record of records) {
      totalCost = totalCost.plus(new Decimal(record.totalCost));
      totalInputTokens += record.inputTokens;
      totalOutputTokens += record.outputTokens;
      totalCachedInputTokens += record.cachedInputTokens;

      executionIds.add(record.executionId);

      if (record.timestamp < minTime) minTime = record.timestamp;
      if (record.timestamp > maxTime) maxTime = record.timestamp;

      // Provider breakdown
      const providerKey = record.provider;
      const existingProvider = providerMap.get(providerKey) || {
        provider: record.provider,
        cost: '0',
        inputTokens: 0,
        outputTokens: 0,
        cachedInputTokens: 0,
        recordCount: 0,
      };

      existingProvider.cost = new Decimal(existingProvider.cost)
        .plus(new Decimal(record.totalCost))
        .toFixed(10);
      existingProvider.inputTokens += record.inputTokens;
      existingProvider.outputTokens += record.outputTokens;
      existingProvider.cachedInputTokens += record.cachedInputTokens;
      existingProvider.recordCount++;

      providerMap.set(providerKey, existingProvider);

      // Model breakdown
      const modelKey = `${record.provider}:${record.model}`;
      const existingModel = modelMap.get(modelKey) || {
        model: record.model,
        provider: record.provider,
        cost: '0',
        inputTokens: 0,
        outputTokens: 0,
        cachedInputTokens: 0,
        recordCount: 0,
      };

      existingModel.cost = new Decimal(existingModel.cost)
        .plus(new Decimal(record.totalCost))
        .toFixed(10);
      existingModel.inputTokens += record.inputTokens;
      existingModel.outputTokens += record.outputTokens;
      existingModel.cachedInputTokens += record.cachedInputTokens;
      existingModel.recordCount++;

      modelMap.set(modelKey, existingModel);
    }

    return {
      scope: agentId,
      scopeType: 'agent',
      agentId,

      totalCost: totalCost.toFixed(10),
      currency,
      recordCount: records.length,
      startTime: minTime,
      endTime: maxTime,

      executionCount: executionIds.size,
      totalInputTokens,
      totalOutputTokens,
      totalCachedInputTokens,

      providerBreakdown: Array.from(providerMap.values()),
      modelBreakdown: Array.from(modelMap.values()),
    };
  }

  /**
   * Aggregate a workflow across agents
   */
  private aggregateWorkflow(workflowId: string, records: CostRecord[]): WorkflowAttribution {
    const currency = records[0].currency;
    let totalCost = new Decimal(0);
    let totalInputTokens = 0;
    let totalOutputTokens = 0;
    let totalCachedInputTokens = 0;
    let minTime = records[0].timestamp;
    let maxTime = records[0].timestamp;

    const agentIds = new Set<string>();
    const executionIds = new Set<string>();
    const providerMap = new Map<string, ProviderBreakdown>();
    const agentMap = new Map<string, AgentBreakdown>();

    for (const record of records) {
      totalCost = totalCost.plus(new Decimal(record.totalCost));
      totalInputTokens += record.inputTokens;
      totalOutputTokens += record.outputTokens;
      totalCachedInputTokens += record.cachedInputTokens;

      agentIds.add(record.agentId);
      executionIds.add(record.executionId);

      if (record.timestamp < minTime) minTime = record.timestamp;
      if (record.timestamp > maxTime) maxTime = record.timestamp;

      // Provider breakdown
      const existingProvider = providerMap.get(record.provider) || {
        provider: record.provider,
        cost: '0',
        inputTokens: 0,
        outputTokens: 0,
        cachedInputTokens: 0,
        recordCount: 0,
      };

      existingProvider.cost = new Decimal(existingProvider.cost)
        .plus(new Decimal(record.totalCost))
        .toFixed(10);
      existingProvider.inputTokens += record.inputTokens;
      existingProvider.outputTokens += record.outputTokens;
      existingProvider.cachedInputTokens += record.cachedInputTokens;
      existingProvider.recordCount++;

      providerMap.set(record.provider, existingProvider);

      // Agent breakdown
      const existingAgent = agentMap.get(record.agentId) || {
        agentId: record.agentId,
        cost: '0',
        executionCount: 0,
        inputTokens: 0,
        outputTokens: 0,
        cachedInputTokens: 0,
      };

      existingAgent.cost = new Decimal(existingAgent.cost)
        .plus(new Decimal(record.totalCost))
        .toFixed(10);
      existingAgent.executionCount++;
      existingAgent.inputTokens += record.inputTokens;
      existingAgent.outputTokens += record.outputTokens;
      existingAgent.cachedInputTokens += record.cachedInputTokens;

      agentMap.set(record.agentId, existingAgent);
    }

    return {
      scope: workflowId,
      scopeType: 'workflow',
      workflowId,

      totalCost: totalCost.toFixed(10),
      currency,
      recordCount: records.length,
      startTime: minTime,
      endTime: maxTime,

      agentCount: agentIds.size,
      executionCount: executionIds.size,
      totalInputTokens,
      totalOutputTokens,
      totalCachedInputTokens,

      agentBreakdown: Array.from(agentMap.values()),
      providerBreakdown: Array.from(providerMap.values()),
    };
  }

  /**
   * Aggregate a tenant across workflows
   */
  private aggregateTenant(tenantId: string, records: CostRecord[]): TenantAttribution {
    const currency = records[0].currency;
    let totalCost = new Decimal(0);
    let totalInputTokens = 0;
    let totalOutputTokens = 0;
    let totalCachedInputTokens = 0;
    let minTime = records[0].timestamp;
    let maxTime = records[0].timestamp;

    const workflowIds = new Set<string>();
    const agentIds = new Set<string>();
    const executionIds = new Set<string>();

    const providerMap = new Map<string, ProviderBreakdown>();
    const agentMap = new Map<string, AgentBreakdown>();
    const workflowMap = new Map<string, WorkflowBreakdown>();

    for (const record of records) {
      totalCost = totalCost.plus(new Decimal(record.totalCost));
      totalInputTokens += record.inputTokens;
      totalOutputTokens += record.outputTokens;
      totalCachedInputTokens += record.cachedInputTokens;

      if (record.workflowId) workflowIds.add(record.workflowId);
      agentIds.add(record.agentId);
      executionIds.add(record.executionId);

      if (record.timestamp < minTime) minTime = record.timestamp;
      if (record.timestamp > maxTime) maxTime = record.timestamp;

      // Provider breakdown
      const existingProvider = providerMap.get(record.provider) || {
        provider: record.provider,
        cost: '0',
        inputTokens: 0,
        outputTokens: 0,
        cachedInputTokens: 0,
        recordCount: 0,
      };

      existingProvider.cost = new Decimal(existingProvider.cost)
        .plus(new Decimal(record.totalCost))
        .toFixed(10);
      existingProvider.inputTokens += record.inputTokens;
      existingProvider.outputTokens += record.outputTokens;
      existingProvider.cachedInputTokens += record.cachedInputTokens;
      existingProvider.recordCount++;

      providerMap.set(record.provider, existingProvider);

      // Agent breakdown
      const existingAgent = agentMap.get(record.agentId) || {
        agentId: record.agentId,
        cost: '0',
        executionCount: 0,
        inputTokens: 0,
        outputTokens: 0,
        cachedInputTokens: 0,
      };

      existingAgent.cost = new Decimal(existingAgent.cost)
        .plus(new Decimal(record.totalCost))
        .toFixed(10);
      existingAgent.executionCount++;
      existingAgent.inputTokens += record.inputTokens;
      existingAgent.outputTokens += record.outputTokens;
      existingAgent.cachedInputTokens += record.cachedInputTokens;

      agentMap.set(record.agentId, existingAgent);

      // Workflow breakdown
      if (record.workflowId) {
        const existingWorkflow = workflowMap.get(record.workflowId) || {
          workflowId: record.workflowId,
          cost: '0',
          agentCount: 0,
          executionCount: 0,
          inputTokens: 0,
          outputTokens: 0,
          cachedInputTokens: 0,
        };

        existingWorkflow.cost = new Decimal(existingWorkflow.cost)
          .plus(new Decimal(record.totalCost))
          .toFixed(10);
        existingWorkflow.executionCount++;
        existingWorkflow.inputTokens += record.inputTokens;
        existingWorkflow.outputTokens += record.outputTokens;
        existingWorkflow.cachedInputTokens += record.cachedInputTokens;

        workflowMap.set(record.workflowId, existingWorkflow);
      }
    }

    // Update agent counts in workflow breakdowns
    for (const [workflowId, workflow] of workflowMap) {
      const workflowRecords = records.filter(r => r.workflowId === workflowId);
      const workflowAgents = new Set(workflowRecords.map(r => r.agentId));
      workflow.agentCount = workflowAgents.size;
    }

    return {
      scope: tenantId,
      scopeType: 'tenant',
      tenantId,

      totalCost: totalCost.toFixed(10),
      currency,
      recordCount: records.length,
      startTime: minTime,
      endTime: maxTime,

      workflowCount: workflowIds.size,
      agentCount: agentIds.size,
      executionCount: executionIds.size,
      totalInputTokens,
      totalOutputTokens,
      totalCachedInputTokens,

      workflowBreakdown: Array.from(workflowMap.values()),
      agentBreakdown: Array.from(agentMap.values()),
      providerBreakdown: Array.from(providerMap.values()),
    };
  }
}
