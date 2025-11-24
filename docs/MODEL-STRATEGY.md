# Model Selection Strategy

**Version**: 2.0
**Date**: 2025-11-24
**Status**: Active Policy

---

## Table of Contents

- [Overview](#overview)
- [Unified Model Approach](#unified-model-approach)
- [Model Comparison](#model-comparison)
- [Implementation](#implementation)
- [Cost Analysis](#cost-analysis)
- [Usage Guidelines](#usage-guidelines)
- [Historical Context](#historical-context)
- [FAQ](#faq)

---

## Overview

The Claude Code Orchestration System uses a **unified model approach** where sub-agents inherit the orchestrator's model by default. This document explains the rationale, implementation, and usage of this strategy.

### Key Principle

**Model inheritance provides flexibility while keeping costs predictable:**

- **Orchestrator**: User selects model via `/model` command based on project needs
- **Sub-Agents**: Automatically inherit orchestrator's model (no explicit specification needed)
- **Result**: Consistent model usage throughout the project with simple control

### Why This Approach

**Version 1.x (pre-v1.15.0)** enforced `model="sonnet"` for all sub-agents to prevent 5x cost overruns when Opus 4.1 was significantly more expensive than Sonnet.

**Version 1.15.0+** removed enforcement because:
- Opus 4.5 pricing is only **1.67x more** than Sonnet (vs 5x for Opus 4.1)
- Opus 4.5 is now marketed as "best for coding" (not just reasoning)
- Enforcement complexity no longer justified for ~67% cost difference
- Model inheritance is simpler and more flexible

---

## Unified Model Approach

### Single Control Point

**Use `/model` command to control all agents:**

```bash
# Sonnet 4.5 throughout (default)
/model sonnet

# Opus 4.5 throughout
/model opus
```

**What happens:**
- Orchestrator uses selected model
- All sub-agents automatically inherit that model
- No explicit model specification needed in Task tool calls

### Sub-Agent Inheritance

When launching sub-agents, **omit the model parameter** to use inheritance:

```python
# Default: Sub-agent inherits orchestrator's model
Task(
    description="Implement authentication service",
    prompt="Read components/auth-service/CLAUDE.md and implement...",
    subagent_type="general-purpose"
    # No model parameter → inherits orchestrator's model
)
```

### Optional Per-Agent Override

In rare cases where specific agents need different models:

```python
# Override: Use specific model for this agent only
Task(
    description="Complex architectural design",
    prompt="Design the authentication architecture...",
    subagent_type="general-purpose",
    model="opus"  # Override inheritance for this specific agent
)
```

**Note**: Per-agent overrides are rarely needed. Most projects should use a single model throughout for consistency.

---

## Model Comparison

### Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

**Released**: September 2025
**Position**: Best coding model, default choice

**Strengths**:
- **Superior coding**: 77.2% on SWE-bench (industry-leading)
- **Long-running coherence**: 30+ hours of autonomous work
- **Cost-effective**: $3 input / $15 output per 1M tokens
- **Fast**: Lower latency than Opus
- **Best value**: Optimal cost/performance balance

**Weaknesses**:
- Slightly lower on pure reasoning tasks (68.9% GPQA Diamond vs 70.6% for Opus)
- May require clearer specifications for complex designs

**Best For**:
- Well-defined specifications
- Cost-sensitive operations
- Long-running autonomous workflows
- Most coding projects (90%+ of use cases)

### Claude Opus 4.5 (claude-opus-4-5-20250929)

**Released**: November 2025
**Position**: Best for coding AND reasoning

**Strengths**:
- **Excellent coding**: Marketed as "best for coding"
- **Best reasoning**: 70.6% on GPQA Diamond (graduate-level science)
- **Design correctness**: Better at high-level architectural decisions
- **Ambiguity handling**: Excels with vague or contradictory specifications

**Weaknesses**:
- Higher cost: $5 input / $25 output per 1M tokens (~1.67x more)
- Higher latency
- May be overkill for well-defined specifications

**Best For**:
- Complex architectural planning
- Ambiguous specification interpretation
- High-stakes design decisions
- Scenarios where reasoning is critical

### Cost Comparison

**Opus 4.5 vs Sonnet 4.5:**
- Input: $5 vs $3 (1.67x)
- Output: $25 vs $15 (1.67x)
- Cached reads: $0.50 vs $0.30 (1.67x)

**Typical project (5 components):**
- Sonnet throughout: ~$2.66
- Opus throughout: ~$4.44
- Difference: **$1.78 (67% increase)**

### Performance Data Sources

- **SWE-bench**: Real-world GitHub issue resolution benchmark
- **GPQA Diamond**: Graduate-level science questions (PhD-level)
- **Pricing**: Anthropic API documentation (November 2025)
- Data current as of January 2025

---

## Implementation

### How It Works

#### 1. User Sets Model

```bash
# Use Sonnet (default, recommended for most projects)
/model sonnet

# Use Opus (for complex/ambiguous specifications)
/model opus
```

#### 2. Orchestrator Launches Sub-Agents

```python
# Recommended: Omit model parameter (uses inheritance)
Task(
    description="Implement authentication service",
    prompt="Read components/auth-service/CLAUDE.md and implement all features...",
    subagent_type="general-purpose"
    # No model parameter → inherits orchestrator's model
)
```

#### 3. All Sub-Agents Use Same Model

- If orchestrator is Sonnet → all sub-agents use Sonnet
- If orchestrator is Opus → all sub-agents use Opus
- Consistent model usage throughout project
- Predictable costs based on single `/model` choice

### Model Parameter Format

The Task tool accepts **generic model names**:

- ✅ **Correct**: `model="sonnet"` (maps to latest Sonnet)
- ✅ **Correct**: `model="opus"` (maps to latest Opus)
- ✅ **Correct**: `model="haiku"` (maps to latest Haiku)
- ❌ **Wrong**: `model="claude-sonnet-4-5-20250929"` (tool doesn't accept this)

The generic names automatically map to the latest version of each model family.

---

## Cost Analysis

### Assumptions

- **Orchestrator**: 100,000 tokens input, 20,000 tokens output (planning, coordination)
- **Sub-Agents** (5 components): 500,000 tokens input each, 100,000 tokens output each (coding)
- **Total**: 2.6M input tokens, 520K output tokens

### Cost Breakdown

#### Scenario 1: Sonnet Throughout (Default)

| Role | Input Tokens | Output Tokens | Input Cost | Output Cost | Total |
|------|-------------|---------------|------------|-------------|-------|
| Orchestrator | 100,000 | 20,000 | $0.30 | $0.30 | $0.60 |
| Sub-Agents (5x) | 2,500,000 | 500,000 | $7.50 | $7.50 | $15.00 |
| **TOTAL** | **2,600,000** | **520,000** | **$7.80** | **$7.80** | **$15.60** |

**Per-component average**: $3.12

#### Scenario 2: Opus Throughout

| Role | Input Tokens | Output Tokens | Input Cost | Output Cost | Total |
|------|-------------|---------------|------------|-------------|-------|
| Orchestrator | 100,000 | 20,000 | $0.50 | $0.50 | $1.00 |
| Sub-Agents (5x) | 2,500,000 | 500,000 | $12.50 | $12.50 | $25.00 |
| **TOTAL** | **2,600,000** | **520,000** | **$13.00** | **$13.00** | **$26.00** |

**Per-component average**: $5.20
**Cost increase vs Scenario 1**: $10.40 (67%)

### Key Insights

1. **Opus costs 1.67x more** but provides better reasoning for complex specs
2. **Cost difference is moderate** (~$10 for typical 5-component project)
3. **Model choice depends on specification complexity**, not just cost
4. **Most projects benefit from Sonnet** (well-defined specs, cost-effective)

---

## Usage Guidelines

### When to Choose Each Model

#### Choose Sonnet 4.5 (Default) When:

✅ **Specifications are well-defined:**
- Detailed API contracts (OpenAPI specs)
- Clear database schemas
- Documented authentication flows
- Explicit test requirements

✅ **Cost optimization is important:**
- Budget-constrained projects
- Multiple components (cost scales)
- Long-running autonomous workflows

✅ **Coding performance is the priority:**
- Implementation-focused work
- Established architectural patterns
- Standard CRUD operations

#### Choose Opus 4.5 When:

✅ **Specifications are complex or ambiguous:**
- Vague requirements ("build a social platform")
- Conflicting specifications
- Novel architectural problems
- Needs design guidance

✅ **Design-level correctness matters more than cost:**
- High-stakes systems
- Complex business logic
- Security-critical applications
- Graduate-level reasoning required

✅ **Budget allows 67% cost increase:**
- ~$10 additional cost for typical project
- Worth it for complex architectural decisions

### Examples

#### Scenario A: Clear Specification → Use Sonnet

```markdown
User provides:
- OpenAPI spec for all endpoints
- Database schema with relationships
- Authentication flow diagram
- Test coverage requirements (80%+)

Recommendation: `/model sonnet`
Rationale: All requirements well-defined, Sonnet excels at implementation
Estimated cost: ~$15 for 5-component project
```

#### Scenario B: Ambiguous Specification → Use Opus

```markdown
User provides:
- High-level description: "Build a social platform for developers"
- No technical details
- Conflicting requirements (real-time + cost-effective)
- Needs architectural guidance

Recommendation: `/model opus`
Rationale: Requires reasoning to clarify design and resolve conflicts
Estimated cost: ~$26 for 5-component project (+$10 vs Sonnet)
```

### Switching Models Mid-Project

You can switch models at any time:

```bash
# Start with Sonnet for implementation
/model sonnet

# Switch to Opus for complex refactoring
/model opus

# Back to Sonnet for routine work
/model sonnet
```

**What happens:**
- Subsequent sub-agents use new model
- Previous sub-agents' work remains unchanged
- Allows dynamic model selection based on current task complexity

---

## Historical Context

### Why Enforcement Was Removed in v1.15.0

#### Version 1.x (Pre-v1.15.0): Enforced Sonnet for Sub-Agents

**Problem**: Opus 4.1 was 5x more expensive than Sonnet ($15 vs $3 input, $75 vs $15 output). If users selected Opus for orchestration, sub-agents would inherit Opus, causing massive cost overruns with no coding benefit.

**Solution**: Enforce `model="sonnet"` in all Task tool invocations to prevent inheritance.

**Implementation**:
```python
# Required in v1.x
Task(
    description="Implement feature",
    prompt="...",
    subagent_type="general-purpose",
    model="sonnet"  # ← REQUIRED to prevent Opus inheritance
)
```

**Result**: Protected against 5x cost overruns, but added enforcement complexity throughout codebase.

#### Version 1.15.0+: Model Inheritance

**Change**: Opus 4.5 released with new pricing:
- Opus 4.5: $5/$25 per MTok (only 1.67x more expensive)
- Opus 4.5 marketed as "best for coding" (not just reasoning)
- Cost/benefit ratio dramatically improved

**Decision**: Remove enforcement, use model inheritance:
- Simplified implementation (no model parameter needed)
- More flexible (easy to switch models)
- Cost difference acceptable (~67% vs 400%)
- Trust users to choose appropriate model

**New Implementation**:
```python
# Recommended in v1.15.0+
Task(
    description="Implement feature",
    prompt="...",
    subagent_type="general-purpose"
    # No model parameter → inherits orchestrator's model
)
```

### Lessons Learned

1. **Enforcement complexity should match cost risk**: 5x cost difference justified enforcement; 1.67x does not
2. **Model capabilities evolve**: Opus 4.5 is better at coding than Opus 4.1
3. **Pricing changes affect architecture**: Policy should adapt to new pricing
4. **Simplicity matters**: Inheritance is easier than enforcement

---

## FAQ

### Q1: Which model should I use?

**A**: Use **Sonnet 4.5** (default) for most projects. Only switch to Opus 4.5 if your specifications are genuinely complex or ambiguous.

### Q2: How much more does Opus cost?

**A**: Opus 4.5 costs ~1.67x more than Sonnet 4.5. For a typical 5-component project:
- Sonnet: ~$15
- Opus: ~$26
- Difference: ~$10 (67% increase)

### Q3: Can I mix models (Opus orchestrator, Sonnet sub-agents)?

**A**: Not by default. Sub-agents inherit the orchestrator's model. To use mixed models, explicitly specify `model="sonnet"` in Task calls (but this is rarely needed and not recommended).

### Q4: What if I start with Sonnet and realize I need Opus?

**A**: Switch models anytime with `/model opus`. Future sub-agents will use Opus. Previous work remains unchanged.

### Q5: Will sub-agents ever use a different model than the orchestrator?

**A**: Only if you explicitly specify `model="X"` in the Task call. By default, they inherit the orchestrator's model.

### Q6: Why not always use Opus if it's "best for coding"?

**A**: Opus is excellent but costs 67% more. For well-defined specifications, Sonnet provides great results at lower cost. Choose based on specification complexity, not just model capabilities.

### Q7: What happens if Anthropic releases new models?

**A**: Generic model names (`model="sonnet"`, `model="opus"`) automatically map to the latest version. No code changes required.

### Q8: Can I force all sub-agents to use Sonnet even if I'm using Opus?

**A**: Yes, explicitly specify `model="sonnet"` in every Task call. But this defeats the purpose of the unified approach and is not recommended in v1.15.0+.

### Q9: How do I check which model I'm using?

**A**: Check your current model selection (method depends on your Claude Code UI). Sub-agents will inherit that model unless overridden.

### Q10: What if I want fine-grained control (different models for different components)?

**A**: Use per-agent overrides:
```python
Task(description="Complex design", ..., model="opus")
Task(description="Routine CRUD", ..., model="sonnet")
```

But this is rarely needed. Most projects benefit from consistent model usage.

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-11-10 | Initial release: Two-tier enforcement strategy |
| 2.0 | 2025-11-24 | **Major update**: Unified model approach (v1.15.0) - removed enforcement, use inheritance |

---

## References

- **Claude API Documentation**: https://docs.anthropic.com/
- **SWE-bench Leaderboard**: https://www.swebench.com/
- **Opus 4.5 Release**: Anthropic blog (November 2025)
- **Task Tool Documentation**: Claude Code built-in tools reference

---

**Maintained by**: Claude Code Orchestration System Team
**Last Updated**: 2025-11-24
**Status**: Active Policy
