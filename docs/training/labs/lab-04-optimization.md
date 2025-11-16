# Lab 4: Cost Optimization

## Overview

Learn advanced cost optimization techniques for LLM operations. Master model selection, caching strategies, prompt optimization, anomaly detection, and A/B testing for cost efficiency.

**Estimated Time:** 90-120 minutes

**Difficulty Level:** Advanced

## Learning Objectives

- Identify cost optimization opportunities
- Implement intelligent model selection strategies
- Configure and monitor caching effectiveness
- Optimize prompts for cost reduction
- Implement batch processing for efficiency
- Configure rate limiting for cost control
- Detect and investigate cost anomalies
- Perform A/B testing for cost-effectiveness
- Create cost optimization reports

## Prerequisites

- [ ] Completed Labs 1-3
- [ ] Understanding of different LLM models and pricing
- [ ] Familiarity with caching concepts
- [ ] Basic statistics knowledge

## Part 1: Cost Analysis and Opportunity Identification

### Step 1.1: Cost Optimization Analyzer

Create `optimization_analyzer.py`:

```python
#!/usr/bin/env python3
"""
Cost Optimization Analyzer
"""

from datetime import datetime, timedelta
from typing import Dict, List, Tuple
from collections import defaultdict
from cost_ops_client import CostOpsClient

class OptimizationAnalyzer:
    """Identify cost optimization opportunities"""

    def __init__(self, base_url: str = "http://localhost:8080"):
        self.client = CostOpsClient(base_url=base_url)

    def analyze_model_costs(self, days: int = 30) -> Dict:
        """Analyze costs by model and identify expensive operations"""
        end_date = datetime.utcnow()
        start_date = end_date - timedelta(days=days)

        costs = self.client.get_costs(
            start_date=start_date.isoformat(),
            end_date=end_date.isoformat()
        )

        # Group by model
        model_stats = defaultdict(lambda: {
            'total_cost': 0.0,
            'request_count': 0,
            'total_tokens': 0,
            'avg_cost_per_request': 0.0,
            'avg_cost_per_1k_tokens': 0.0
        })

        for cost in costs:
            model = cost['model']
            cost_value = float(cost['total_cost'])

            model_stats[model]['total_cost'] += cost_value
            model_stats[model]['request_count'] += 1

        # Calculate averages
        for model, stats in model_stats.items():
            if stats['request_count'] > 0:
                stats['avg_cost_per_request'] = stats['total_cost'] / stats['request_count']

        return dict(model_stats)

    def identify_expensive_models(self, model_stats: Dict, threshold: float = 0.50) -> List[Dict]:
        """Identify models consuming more than threshold of total budget"""
        total_cost = sum(stats['total_cost'] for stats in model_stats.values())

        expensive_models = []
        for model, stats in model_stats.items():
            percentage = (stats['total_cost'] / total_cost * 100) if total_cost > 0 else 0

            if percentage > threshold:
                expensive_models.append({
                    'model': model,
                    'cost': stats['total_cost'],
                    'percentage': percentage,
                    'requests': stats['request_count'],
                    'avg_cost': stats['avg_cost_per_request']
                })

        # Sort by cost
        expensive_models.sort(key=lambda x: x['cost'], reverse=True)
        return expensive_models

    def suggest_model_alternatives(self, model: str) -> List[Dict]:
        """Suggest cheaper alternative models"""

        # Model alternatives mapping (based on capabilities)
        alternatives = {
            'gpt-4': [
                {'model': 'gpt-4-turbo', 'savings_percent': 50, 'reason': '50% cheaper, similar quality'},
                {'model': 'gpt-3.5-turbo', 'savings_percent': 95, 'reason': '95% cheaper for simpler tasks'},
                {'model': 'claude-3-sonnet-20240229', 'savings_percent': 70, 'reason': '70% cheaper, comparable quality'}
            ],
            'gpt-4-turbo': [
                {'model': 'gpt-3.5-turbo', 'savings_percent': 90, 'reason': '90% cheaper for routine tasks'},
                {'model': 'claude-3-haiku', 'savings_percent': 95, 'reason': '95% cheaper for fast tasks'}
            ],
            'claude-3-opus-20240229': [
                {'model': 'claude-3-sonnet-20240229', 'savings_percent': 80, 'reason': '80% cheaper, still high quality'},
                {'model': 'claude-3-haiku', 'savings_percent': 95, 'reason': '95% cheaper for routine work'}
            ],
            'claude-3-sonnet-20240229': [
                {'model': 'claude-3-haiku', 'savings_percent': 90, 'reason': '90% cheaper for simpler tasks'},
                {'model': 'gpt-3.5-turbo', 'savings_percent': 80, 'reason': '80% cheaper alternative'}
            ]
        }

        return alternatives.get(model, [])

    def analyze_caching_opportunities(self, days: int = 30) -> Dict:
        """Identify caching optimization opportunities"""
        end_date = datetime.utcnow()
        start_date = end_date - timedelta(days=days)

        costs = self.client.get_costs(
            start_date=start_date.isoformat(),
            end_date=end_date.isoformat()
        )

        # Group by provider
        provider_analysis = defaultdict(lambda: {
            'total_cost': 0.0,
            'cached_cost': 0.0,
            'requests': 0,
            'cached_requests': 0
        })

        for cost in costs:
            provider = cost['provider']
            cost_value = float(cost['total_cost'])

            provider_analysis[provider]['total_cost'] += cost_value
            provider_analysis[provider]['requests'] += 1

            # Check if cached tokens were used (Anthropic)
            if provider == 'anthropic' and float(cost.get('cached_cost', 0)) > 0:
                provider_analysis[provider]['cached_cost'] += float(cost['cached_cost'])
                provider_analysis[provider]['cached_requests'] += 1

        # Calculate potential savings
        opportunities = {}
        for provider, analysis in provider_analysis.items():
            if provider == 'anthropic':
                cache_hit_rate = (analysis['cached_requests'] / analysis['requests'] * 100) if analysis['requests'] > 0 else 0
                potential_savings = analysis['total_cost'] * 0.5 * (1 - cache_hit_rate / 100)

                opportunities[provider] = {
                    'current_cache_hit_rate': cache_hit_rate,
                    'current_savings': analysis['cached_cost'],
                    'potential_additional_savings': potential_savings,
                    'recommendation': 'Enable prompt caching' if cache_hit_rate < 50 else 'Optimize cache usage'
                }

        return opportunities

    def detect_cost_anomalies(self, days: int = 30, threshold: float = 2.0) -> List[Dict]:
        """Detect unusual cost spikes"""
        end_date = datetime.utcnow()
        start_date = end_date - timedelta(days=days)

        costs = self.client.get_costs(
            start_date=start_date.isoformat(),
            end_date=end_date.isoformat()
        )

        # Group by day
        daily_costs = defaultdict(float)
        for cost in costs:
            timestamp = datetime.fromisoformat(cost['timestamp'].replace('Z', '+00:00'))
            date_key = timestamp.strftime('%Y-%m-%d')
            daily_costs[date_key] += float(cost['total_cost'])

        # Calculate statistics
        costs_list = list(daily_costs.values())
        if len(costs_list) < 2:
            return []

        mean_cost = sum(costs_list) / len(costs_list)
        variance = sum((x - mean_cost) ** 2 for x in costs_list) / len(costs_list)
        std_dev = variance ** 0.5

        # Find anomalies (Z-score method)
        anomalies = []
        for date, cost in daily_costs.items():
            z_score = (cost - mean_cost) / std_dev if std_dev > 0 else 0

            if abs(z_score) > threshold:
                anomalies.append({
                    'date': date,
                    'cost': cost,
                    'mean_cost': mean_cost,
                    'z_score': z_score,
                    'deviation_percent': ((cost - mean_cost) / mean_cost * 100) if mean_cost > 0 else 0
                })

        # Sort by z_score
        anomalies.sort(key=lambda x: abs(x['z_score']), reverse=True)
        return anomalies

    def generate_optimization_report(self):
        """Generate comprehensive optimization report"""
        print("=" * 80)
        print("COST OPTIMIZATION REPORT")
        print("=" * 80)

        # Model analysis
        print("\n📊 MODEL COST ANALYSIS")
        print("-" * 80)

        model_stats = self.analyze_model_costs(days=30)
        expensive_models = self.identify_expensive_models(model_stats, threshold=20.0)

        if expensive_models:
            print(f"\n🔴 High-Cost Models (>20% of budget):")
            for model_info in expensive_models:
                print(f"\n  Model: {model_info['model']}")
                print(f"  Cost: ${model_info['cost']:,.2f} ({model_info['percentage']:.1f}% of total)")
                print(f"  Requests: {model_info['requests']}")
                print(f"  Avg/Request: ${model_info['avg_cost']:.4f}")

                # Suggest alternatives
                alternatives = self.suggest_model_alternatives(model_info['model'])
                if alternatives:
                    print(f"\n  💡 Suggested Alternatives:")
                    for alt in alternatives[:2]:  # Show top 2
                        potential_savings = model_info['cost'] * (alt['savings_percent'] / 100)
                        print(f"    • {alt['model']}: {alt['reason']}")
                        print(f"      Potential savings: ${potential_savings:,.2f}/month")
        else:
            print("  ✓ No high-cost models identified")

        # Caching analysis
        print("\n\n🗄️  CACHING OPTIMIZATION")
        print("-" * 80)

        caching_ops = self.analyze_caching_opportunities(days=30)

        if caching_ops:
            for provider, analysis in caching_ops.items():
                print(f"\n  Provider: {provider}")
                print(f"  Cache Hit Rate: {analysis['current_cache_hit_rate']:.1f}%")
                print(f"  Current Savings: ${analysis['current_savings']:,.2f}")
                print(f"  Potential Additional: ${analysis['potential_additional_savings']:,.2f}")
                print(f"  💡 Recommendation: {analysis['recommendation']}")
        else:
            print("  No caching opportunities identified")

        # Anomaly detection
        print("\n\n🚨 COST ANOMALIES")
        print("-" * 80)

        anomalies = self.detect_cost_anomalies(days=30, threshold=2.0)

        if anomalies:
            print(f"\n  Found {len(anomalies)} unusual cost patterns:")
            for anomaly in anomalies[:5]:  # Show top 5
                print(f"\n  Date: {anomaly['date']}")
                print(f"  Cost: ${anomaly['cost']:,.2f} (vs avg ${anomaly['mean_cost']:,.2f})")
                print(f"  Deviation: {anomaly['deviation_percent']:+.1f}%")
                print(f"  Z-Score: {anomaly['z_score']:.2f}")
        else:
            print("  ✓ No significant cost anomalies detected")

        # Summary
        print("\n\n📋 OPTIMIZATION SUMMARY")
        print("-" * 80)

        total_potential_savings = 0.0

        # Calculate from model alternatives
        for model_info in expensive_models[:3]:  # Top 3 expensive
            alternatives = self.suggest_model_alternatives(model_info['model'])
            if alternatives:
                savings = model_info['cost'] * (alternatives[0]['savings_percent'] / 100)
                total_potential_savings += savings

        # Calculate from caching
        for provider, analysis in caching_ops.items():
            total_potential_savings += analysis['potential_additional_savings']

        print(f"\n  Total Potential Monthly Savings: ${total_potential_savings:,.2f}")

        print("\n  Top Recommendations:")
        print("  1. Review high-cost models and consider alternatives")
        print("  2. Enable prompt caching where available")
        print("  3. Investigate cost anomalies")
        print("  4. Implement batch processing for similar requests")
        print("  5. Set up rate limiting to prevent cost spikes")

        print("\n" + "=" * 80)


if __name__ == "__main__":
    analyzer = OptimizationAnalyzer()
    analyzer.generate_optimization_report()
```

Run the analyzer:

```bash
python optimization_analyzer.py
```

## Part 2: Intelligent Model Selection

Implement automatic model selection based on cost and requirements:

```python
#!/usr/bin/env python3
"""
Intelligent Model Selector
"""

from typing import Dict, List, Optional
from dataclasses import dataclass

@dataclass
class ModelCapability:
    """Model capabilities and characteristics"""
    name: str
    provider: str
    cost_per_1m_input: float
    cost_per_1m_output: float
    context_window: int
    max_output: int
    quality_score: float  # 1-10
    speed_score: float    # 1-10
    supports_caching: bool
    best_for: List[str]

class ModelSelector:
    """Intelligent model selection based on requirements"""

    def __init__(self):
        # Model database
        self.models = [
            ModelCapability(
                name="gpt-4-turbo",
                provider="openai",
                cost_per_1m_input=10.0,
                cost_per_1m_output=30.0,
                context_window=128000,
                max_output=4096,
                quality_score=9.5,
                speed_score=7.0,
                supports_caching=False,
                best_for=["complex reasoning", "code generation", "analysis"]
            ),
            ModelCapability(
                name="gpt-3.5-turbo",
                provider="openai",
                cost_per_1m_input=0.5,
                cost_per_1m_output=1.5,
                context_window=16385,
                max_output=4096,
                quality_score=7.5,
                speed_score=9.0,
                supports_caching=False,
                best_for=["simple tasks", "classification", "summarization"]
            ),
            ModelCapability(
                name="claude-3-opus-20240229",
                provider="anthropic",
                cost_per_1m_input=15.0,
                cost_per_1m_output=75.0,
                context_window=200000,
                max_output=4096,
                quality_score=9.8,
                speed_score=6.0,
                supports_caching=True,
                best_for=["complex tasks", "long documents", "creative writing"]
            ),
            ModelCapability(
                name="claude-3-sonnet-20240229",
                provider="anthropic",
                cost_per_1m_input=3.0,
                cost_per_1m_output=15.0,
                context_window=200000,
                max_output=4096,
                quality_score=8.5,
                speed_score=8.0,
                supports_caching=True,
                best_for=["balanced tasks", "analysis", "content generation"]
            ),
            ModelCapability(
                name="claude-3-haiku",
                provider="anthropic",
                cost_per_1m_input=0.25,
                cost_per_1m_output=1.25,
                context_window=200000,
                max_output=4096,
                quality_score=7.0,
                speed_score=9.5,
                supports_caching=True,
                best_for=["simple tasks", "fast responses", "high volume"]
            ),
            ModelCapability(
                name="gemini-pro",
                provider="google",
                cost_per_1m_input=0.5,
                cost_per_1m_output=1.5,
                context_window=32000,
                max_output=8192,
                quality_score=8.0,
                speed_score=8.5,
                supports_caching=False,
                best_for=["multimodal", "general purpose", "cost-effective"]
            )
        ]

    def select_model(
        self,
        task_type: str,
        budget_per_request: Optional[float] = None,
        min_quality: Optional[float] = None,
        min_speed: Optional[float] = None,
        required_context: Optional[int] = None,
        prefer_caching: bool = False
    ) -> List[ModelCapability]:
        """Select best models based on requirements"""

        candidates = []

        for model in self.models:
            # Filter by task type
            if task_type and task_type not in model.best_for:
                continue

            # Filter by quality requirement
            if min_quality and model.quality_score < min_quality:
                continue

            # Filter by speed requirement
            if min_speed and model.speed_score < min_speed:
                continue

            # Filter by context requirement
            if required_context and model.context_window < required_context:
                continue

            # Filter by caching preference
            if prefer_caching and not model.supports_caching:
                continue

            # Estimate cost for typical request
            # Assume 1000 input, 500 output tokens
            est_cost = (model.cost_per_1m_input * 1000 / 1_000_000) + \
                       (model.cost_per_1m_output * 500 / 1_000_000)

            # Filter by budget
            if budget_per_request and est_cost > budget_per_request:
                continue

            candidates.append(model)

        # Sort by cost-effectiveness (quality / cost)
        candidates.sort(
            key=lambda m: m.quality_score / ((m.cost_per_1m_input + m.cost_per_1m_output) / 2),
            reverse=True
        )

        return candidates

    def compare_models(self, prompt_tokens: int, completion_tokens: int) -> List[Dict]:
        """Compare all models for a specific workload"""

        comparisons = []

        for model in self.models:
            input_cost = (model.cost_per_1m_input * prompt_tokens) / 1_000_000
            output_cost = (model.cost_per_1m_output * completion_tokens) / 1_000_000
            total_cost = input_cost + output_cost

            # Apply caching discount if available (50% for cached)
            cached_cost = total_cost * 0.5 if model.supports_caching else total_cost

            comparisons.append({
                'model': model.name,
                'provider': model.provider,
                'cost': total_cost,
                'cost_with_caching': cached_cost,
                'savings_with_caching': total_cost - cached_cost,
                'quality_score': model.quality_score,
                'speed_score': model.speed_score,
                'cost_per_quality': total_cost / model.quality_score
            })

        # Sort by cost
        comparisons.sort(key=lambda x: x['cost'])

        return comparisons

    def display_recommendations(self, task_type: str, constraints: Dict = None):
        """Display model recommendations"""
        constraints = constraints or {}

        print("=" * 80)
        print(f"MODEL RECOMMENDATIONS FOR: {task_type.upper()}")
        print("=" * 80)

        recommendations = self.select_model(task_type=task_type, **constraints)

        if not recommendations:
            print("\n⚠️  No models match the specified constraints")
            print("Try relaxing quality or speed requirements")
            return

        print(f"\nFound {len(recommendations)} suitable models:")
        print("\n" + "-" * 80)

        for idx, model in enumerate(recommendations[:5], 1):
            print(f"\n{idx}. {model.name} ({model.provider})")
            print(f"   Quality: {'⭐' * int(model.quality_score)}")
            print(f"   Speed: {'⚡' * int(model.speed_score)}")
            print(f"   Cost: ${model.cost_per_1m_input}/M in, ${model.cost_per_1m_output}/M out")
            print(f"   Context: {model.context_window:,} tokens")
            print(f"   Caching: {'✓' if model.supports_caching else '✗'}")
            print(f"   Best for: {', '.join(model.best_for)}")

        print("\n" + "=" * 80)


if __name__ == "__main__":
    selector = ModelSelector()

    # Example 1: Simple task with budget constraint
    print("\nExample 1: Simple task, budget-conscious")
    selector.display_recommendations("simple tasks", {
        'budget_per_request': 0.01
    })

    # Example 2: Complex task requiring high quality
    print("\n\nExample 2: Complex analysis, high quality required")
    selector.display_recommendations("analysis", {
        'min_quality': 8.5
    })

    # Example 3: High volume with caching
    print("\n\nExample 3: High volume, caching preferred")
    selector.display_recommendations("summarization", {
        'prefer_caching': True,
        'min_speed': 8.0
    })

    # Example 4: Cost comparison
    print("\n\n" + "=" * 80)
    print("COST COMPARISON (1000 input, 500 output tokens)")
    print("=" * 80)

    comparisons = selector.compare_models(1000, 500)

    print(f"\n{'Model':<30} {'Cost':<12} {'With Cache':<12} {'Quality':<10}")
    print("-" * 80)

    for comp in comparisons:
        print(f"{comp['model']:<30} ${comp['cost']:<11.6f} ${comp['cost_with_caching']:<11.6f} {comp['quality_score']:<10.1f}")
```

Run the model selector:

```bash
python model_selector.py
```

## Part 3: Prompt Optimization for Cost Reduction

Optimize prompts to reduce token usage:

```python
#!/usr/bin/env python3
"""
Prompt Optimization for Cost Reduction
"""

from typing import Dict, List

class PromptOptimizer:
    """Optimize prompts to reduce token usage and costs"""

    def analyze_prompt(self, prompt: str) -> Dict:
        """Analyze prompt for optimization opportunities"""

        # Estimate token count (rough approximation: 1 token ≈ 4 chars)
        estimated_tokens = len(prompt) / 4

        issues = []
        suggestions = []

        # Check for verbosity
        if len(prompt.split()) > 200:
            issues.append("Very long prompt")
            suggestions.append("Consider condensing instructions")

        # Check for redundancy
        words = prompt.lower().split()
        if len(words) != len(set(words)):
            issues.append("Contains repeated words")
            suggestions.append("Remove redundant information")

        # Check for examples
        if "example:" in prompt.lower() or "for instance" in prompt.lower():
            example_count = prompt.lower().count("example")
            if example_count > 3:
                issues.append(f"Contains {example_count} examples")
                suggestions.append("Reduce to 1-2 essential examples")

        # Check for formatting
        if "\n\n\n" in prompt:
            issues.append("Excessive whitespace")
            suggestions.append("Remove extra newlines")

        return {
            'original_length': len(prompt),
            'estimated_tokens': estimated_tokens,
            'word_count': len(prompt.split()),
            'issues': issues,
            'suggestions': suggestions
        }

    def optimize_prompt(self, prompt: str) -> str:
        """Apply automated optimizations"""

        # Remove excessive whitespace
        optimized = ' '.join(prompt.split())

        # Replace verbose phrases
        replacements = {
            'in order to': 'to',
            'due to the fact that': 'because',
            'at this point in time': 'now',
            'for the purpose of': 'for',
            'in the event that': 'if',
        }

        for verbose, concise in replacements.items():
            optimized = optimized.replace(verbose, concise)

        return optimized

    def compare_costs(self, original: str, optimized: str, model: str = "gpt-4-turbo"):
        """Compare costs between original and optimized prompts"""

        # Token estimates
        original_tokens = len(original) / 4
        optimized_tokens = len(optimized) / 4

        # Cost rates (per million tokens)
        rates = {
            'gpt-4-turbo': 10.0,
            'gpt-3.5-turbo': 0.5,
            'claude-3-sonnet-20240229': 3.0
        }

        rate = rates.get(model, 10.0)

        # Calculate costs (assuming 1000 requests)
        requests = 1000
        original_cost = (original_tokens * requests * rate) / 1_000_000
        optimized_cost = (optimized_tokens * requests * rate) / 1_000_000

        savings = original_cost - optimized_cost
        savings_percent = (savings / original_cost * 100) if original_cost > 0 else 0

        return {
            'original_tokens': original_tokens,
            'optimized_tokens': optimized_tokens,
            'token_reduction': original_tokens - optimized_tokens,
            'token_reduction_percent': ((original_tokens - optimized_tokens) / original_tokens * 100) if original_tokens > 0 else 0,
            'original_cost': original_cost,
            'optimized_cost': optimized_cost,
            'savings': savings,
            'savings_percent': savings_percent,
            'requests': requests
        }


# Example usage
if __name__ == "__main__":
    optimizer = PromptOptimizer()

    # Example prompt
    original_prompt = """
    Please analyze the following text and provide a comprehensive summary.
    In order to create the summary, you should first read through the entire text.
    After reading, identify the main points and key takeaways.
    Then, condense these points into a concise summary that captures the essence
    of the original text while being significantly shorter in length.

    For example, if the text is about climate change, you might summarize the
    causes, effects, and potential solutions.

    For instance, a good summary would include the main arguments presented
    and the conclusions drawn by the author.

    Example: "The text discusses climate change, highlighting rising temperatures
    and suggesting renewable energy as a solution."

    Please ensure your summary is clear and well-structured.
    """

    print("=" * 80)
    print("PROMPT OPTIMIZATION ANALYSIS")
    print("=" * 80)

    # Analyze
    analysis = optimizer.analyze_prompt(original_prompt)

    print(f"\n📊 Original Prompt Analysis:")
    print(f"  Length: {analysis['original_length']} characters")
    print(f"  Estimated Tokens: {analysis['estimated_tokens']:.0f}")
    print(f"  Word Count: {analysis['word_count']}")

    if analysis['issues']:
        print(f"\n⚠️  Issues Found:")
        for issue in analysis['issues']:
            print(f"  • {issue}")

        print(f"\n💡 Suggestions:")
        for suggestion in analysis['suggestions']:
            print(f"  • {suggestion}")

    # Optimize
    optimized_prompt = optimizer.optimize_prompt(original_prompt)

    print(f"\n✨ Optimized Prompt:")
    print("-" * 80)
    print(optimized_prompt)
    print("-" * 80)

    # Compare costs
    print(f"\n💰 Cost Comparison (1000 requests):")
    comparison = optimizer.compare_costs(original_prompt, optimized_prompt)

    print(f"  Original: {comparison['original_tokens']:.0f} tokens → ${comparison['original_cost']:.4f}")
    print(f"  Optimized: {comparison['optimized_tokens']:.0f} tokens → ${comparison['optimized_cost']:.4f}")
    print(f"  Savings: {comparison['token_reduction']:.0f} tokens ({comparison['token_reduction_percent']:.1f}%)")
    print(f"  Cost Savings: ${comparison['savings']:.4f} ({comparison['savings_percent']:.1f}%)")

    print("\n" + "=" * 80)
```

## Part 4: A/B Testing for Cost Effectiveness

Implement A/B testing to find the most cost-effective approach:

```python
#!/usr/bin/env python3
"""
A/B Testing for Cost Optimization
"""

from datetime import datetime
from typing import Dict, List
import random
from dataclasses import dataclass

@dataclass
class ABTestResult:
    """Results from A/B test"""
    variant: str
    requests: int
    total_cost: float
    avg_cost: float
    quality_score: float
    latency_ms: float

class CostABTester:
    """A/B testing for cost optimization"""

    def __init__(self):
        self.results = {}

    def run_test(
        self,
        test_name: str,
        variant_a: Dict,
        variant_b: Dict,
        sample_size: int = 100
    ):
        """Run A/B test comparing two approaches"""

        print("=" * 80)
        print(f"A/B TEST: {test_name}")
        print("=" * 80)

        print(f"\nVariant A: {variant_a['name']}")
        print(f"  Model: {variant_a['model']}")
        print(f"  Approach: {variant_a['approach']}")

        print(f"\nVariant B: {variant_b['name']}")
        print(f"  Model: {variant_b['model']}")
        print(f"  Approach: {variant_b['approach']}")

        print(f"\nRunning test with {sample_size} requests per variant...")

        # Simulate results (in real implementation, make actual API calls)
        results_a = self._simulate_variant(variant_a, sample_size)
        results_b = self._simulate_variant(variant_b, sample_size)

        # Calculate statistics
        print("\n" + "-" * 80)
        print("RESULTS")
        print("-" * 80)

        self._display_results("Variant A", results_a)
        self._display_results("Variant B", results_b)

        # Compare
        print("\n" + "-" * 80)
        print("COMPARISON")
        print("-" * 80)

        cost_diff = results_b.avg_cost - results_a.avg_cost
        cost_diff_percent = (cost_diff / results_a.avg_cost * 100) if results_a.avg_cost > 0 else 0

        quality_diff = results_b.quality_score - results_a.quality_score
        latency_diff = results_b.latency_ms - results_a.latency_ms

        print(f"\nCost Difference:")
        print(f"  Variant B vs A: ${cost_diff:+.6f} ({cost_diff_percent:+.1f}%)")

        print(f"\nQuality Difference:")
        print(f"  Variant B vs A: {quality_diff:+.2f} points")

        print(f"\nLatency Difference:")
        print(f"  Variant B vs A: {latency_diff:+.0f}ms")

        # Recommendation
        print("\n" + "-" * 80)
        print("RECOMMENDATION")
        print("-" * 80)

        if cost_diff < 0 and quality_diff >= -0.5:  # B is cheaper and quality is similar
            print("\n✓ Use Variant B")
            print(f"  Saves ${abs(cost_diff):.6f} per request")
            print(f"  Projected monthly savings (10K requests): ${abs(cost_diff) * 10000:.2f}")
        elif quality_diff > 1.0 and cost_diff_percent < 20:  # B is significantly better quality
            print("\n✓ Use Variant B")
            print(f"  Better quality ({quality_diff:+.1f} points)")
            print(f"  Acceptable cost increase: {cost_diff_percent:.1f}%")
        else:
            print("\n✓ Use Variant A")
            print(f"  Better cost-to-quality ratio")

        print("\n" + "=" * 80)

    def _simulate_variant(self, variant: Dict, requests: int) -> ABTestResult:
        """Simulate variant performance (replace with actual API calls)"""

        # Model cost characteristics
        model_costs = {
            'gpt-4-turbo': {'base': 0.02, 'variance': 0.005},
            'gpt-3.5-turbo': {'base': 0.001, 'variance': 0.0002},
            'claude-3-sonnet-20240229': {'base': 0.008, 'variance': 0.002},
            'claude-3-haiku': {'base': 0.0005, 'variance': 0.0001}
        }

        model_quality = {
            'gpt-4-turbo': 9.0,
            'gpt-3.5-turbo': 7.5,
            'claude-3-sonnet-20240229': 8.5,
            'claude-3-haiku': 7.0
        }

        cost_config = model_costs.get(variant['model'], {'base': 0.01, 'variance': 0.002})
        base_quality = model_quality.get(variant['model'], 8.0)

        # Simulate requests
        total_cost = 0.0
        for _ in range(requests):
            # Add random variance
            cost = cost_config['base'] + random.uniform(-cost_config['variance'], cost_config['variance'])
            total_cost += cost

        avg_cost = total_cost / requests
        quality = base_quality + random.uniform(-0.5, 0.5)
        latency = random.uniform(1000, 3000)

        return ABTestResult(
            variant=variant['name'],
            requests=requests,
            total_cost=total_cost,
            avg_cost=avg_cost,
            quality_score=quality,
            latency_ms=latency
        )

    def _display_results(self, name: str, results: ABTestResult):
        """Display test results"""
        print(f"\n{name}:")
        print(f"  Requests: {results.requests}")
        print(f"  Total Cost: ${results.total_cost:.6f}")
        print(f"  Avg Cost: ${results.avg_cost:.6f}")
        print(f"  Quality Score: {results.quality_score:.2f}/10")
        print(f"  Avg Latency: {results.latency_ms:.0f}ms")


# Example usage
if __name__ == "__main__":
    tester = CostABTester()

    # Test 1: GPT-4 Turbo vs GPT-3.5 Turbo
    tester.run_test(
        "Simple Classification Task",
        variant_a={
            'name': 'GPT-4 Turbo',
            'model': 'gpt-4-turbo',
            'approach': 'High quality, standard prompt'
        },
        variant_b={
            'name': 'GPT-3.5 Turbo',
            'model': 'gpt-3.5-turbo',
            'approach': 'Fast, cost-effective'
        },
        sample_size=100
    )

    # Test 2: Standard vs Cached approach
    tester.run_test(
        "Document Analysis with Caching",
        variant_a={
            'name': 'Standard Approach',
            'model': 'claude-3-sonnet-20240229',
            'approach': 'No caching'
        },
        variant_b={
            'name': 'Cached Approach',
            'model': 'claude-3-sonnet-20240229',
            'approach': 'With prompt caching'
        },
        sample_size=100
    )
```

## Exercises and Challenges

### Exercise 1: Build a Cost Router
Create a system that automatically routes requests to the most cost-effective model based on task complexity.

### Exercise 2: Caching Strategy
Implement a caching strategy simulator that shows potential savings from different cache hit rates.

### Exercise 3: Batch Processing Optimizer
Build a batch processing system that groups similar requests to reduce costs.

### Exercise 4: Cost Anomaly Alerting
Create an automated alerting system that notifies when costs deviate from expected patterns.

### Exercise 5: Optimization Dashboard
Build a comprehensive dashboard showing all optimization opportunities and their potential impact.

## Review Questions

1. What are the main factors to consider when selecting an LLM model?
2. How can prompt caching reduce costs?
3. What is the benefit of A/B testing for cost optimization?
4. How do you calculate ROI on optimization efforts?
5. What metrics indicate a cost anomaly?

## Next Steps

Continue to **Lab 5: Enterprise Integration** to learn about:
- Multi-tenant setup
- SSO integration
- RBAC configuration
- Production deployment

---

**End of Lab 4**
