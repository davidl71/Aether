# Agent Delegation & Model Selection Automation Plan

**Date**: 2025-11-20
**Purpose**: Automate intelligent agent delegation and model selection based on task characteristics

---

## Overview

This automation determines:
1. **Scope**: Where should the agent run? (local/worktree/cloud)
2. **Model**: Which model/composer should be used? (auto/composer1/etc.)
3. **Agent Type**: Which specialized agent should handle this? (backend/web/ipad/etc.)

---

## Problem Statement

Currently, agent delegation and model selection is manual. We need to automate:
- **Scope Detection**: Determine if task needs local files, worktree isolation, or cloud resources
- **Model Selection**: Choose optimal model based on task complexity, language, and requirements
- **Agent Routing**: Route tasks to appropriate specialized agents

---

## Tractatus Analysis: What is Agent Delegation?

**Agent Delegation = Task Analysis × Scope Detection × Model Selection × Agent Routing × Resource Availability**

### Components:

1. **Task Analysis**: Understand task requirements, complexity, dependencies
2. **Scope Detection**: Determine execution environment (local/worktree/cloud)
3. **Model Selection**: Choose optimal AI model (auto/composer1/claude/etc.)
4. **Agent Routing**: Route to specialized agent (backend/web/ipad/etc.)
5. **Resource Availability**: Check available resources (compute, storage, network)

---

## Sequential Planning: How to Automate Delegation?

### Step 1: Task Classification
- Analyze task description and requirements
- Identify task type (code/design/debug/research)
- Determine complexity level
- Extract language/technology requirements

### Step 2: Scope Detection
- **Local**: File operations, quick edits, local testing
- **Worktree**: Isolated changes, feature branches, parallel work
- **Cloud**: Heavy compute, large datasets, shared resources

### Step 3: Model Selection
- **Auto**: Let system decide (default)
- **Composer1**: Complex multi-file changes, architecture
- **Claude**: Code review, documentation, analysis
- **Specialized**: Language-specific models if available

### Step 4: Agent Routing
- **Backend**: Rust, C++, Python backend work
- **Web**: React, TypeScript, frontend
- **iPad**: Swift, SwiftUI, iOS
- **Desktop**: Swift, macOS app
- **TUI**: C++ TUI, terminal interfaces
- **Data**: Data pipelines, analysis

### Step 5: Execution Planning
- Generate execution plan
- Check resource availability
- Create delegation task
- Monitor execution

---

## Scope Detection Rules

### Local Scope
**Use when:**
- Quick file edits (< 5 files)
- Simple refactoring
- Documentation updates
- Configuration changes
- Single-component changes
- No isolation needed

**Indicators:**
- Task mentions "update", "fix", "add" (simple)
- Single file or small scope
- No branch/feature context
- Quick turnaround expected

### Worktree Scope
**Use when:**
- Feature development
- Large refactoring (> 5 files)
- Parallel work streams
- Branch-specific changes
- Isolation required
- Multi-component changes

**Indicators:**
- Task mentions "feature", "implement", "refactor"
- Multiple files/components
- Requires branch context
- Long-running work
- Needs isolation

### Cloud Scope
**Use when:**
- Heavy computation
- Large dataset processing
- Shared resources needed
- CI/CD integration
- Distributed work
- Resource-intensive tasks

**Indicators:**
- Task mentions "analysis", "processing", "compute"
- Large data operations
- Requires shared infrastructure
- CI/CD related
- Resource-intensive

---

## Model Selection Rules

### Auto (Default)
**Use when:**
- Standard coding tasks
- Well-defined requirements
- No special needs
- General purpose work

### Composer1
**Use when:**
- Complex multi-file changes
- Architecture decisions
- Cross-component refactoring
- Large-scale changes
- Requires deep understanding

**Indicators:**
- Task mentions "architecture", "refactor", "redesign"
- Multiple components affected
- Requires system understanding
- Complex dependencies

### Claude (or specific model)
**Use when:**
- Code review needed
- Documentation writing
- Analysis tasks
- Research tasks
- High-quality output critical

**Indicators:**
- Task mentions "review", "document", "analyze", "research"
- Quality over speed
- Requires reasoning
- Complex analysis

---

## Agent Routing Rules

### Backend Agent
**Route when:**
- Rust code (`agents/backend/`)
- C++ core (`native/src/`)
- Python backend (`python/`)
- API development
- Server-side logic

**Keywords**: rust, c++, python, backend, api, server, service

### Web Agent
**Route when:**
- React/TypeScript (`web/src/`)
- Frontend components
- UI/UX work
- Web SPA features

**Keywords**: react, typescript, web, frontend, ui, spa, component

### iPad Agent
**Route when:**
- Swift/SwiftUI (`ios/` or iPad-specific)
- iOS development
- iPad-specific features

**Keywords**: swift, swiftui, ios, ipad, apple

### Desktop Agent
**Route when:**
- macOS app (`desktop/`)
- Desktop-specific features
- AppKit development

**Keywords**: desktop, macos, appkit, app

### TUI Agent
**Route when:**
- C++ TUI (`native/src/tui/`)
- Terminal interfaces
- CLI improvements

**Keywords**: tui, terminal, cli, c++

### Data Agent
**Route when:**
- Data pipelines
- Analysis scripts
- Data processing

**Keywords**: data, pipeline, analysis, process

---

## Implementation Strategy

### Phase 1: Task Classification (Week 1)
1. Create task classifier
2. Extract task characteristics
3. Identify task type and complexity
4. Generate classification metadata

### Phase 2: Scope Detection (Week 1)
1. Implement scope detection rules
2. Analyze file/dependency patterns
3. Determine optimal scope
4. Generate scope recommendation

### Phase 3: Model Selection (Week 2)
1. Implement model selection rules
2. Consider task complexity
3. Check model availability
4. Generate model recommendation

### Phase 4: Agent Routing (Week 2)
1. Implement agent routing rules
2. Match tasks to agents
3. Check agent availability
4. Generate routing recommendation

### Phase 5: Integration (Week 3)
1. Integrate with Todo2
2. Create delegation automation
3. Monitor and learn
4. Refine rules based on results

---

## Automation Script Structure

```python
class AgentDelegationAutomation(IntelligentAutomationBase):
    def _get_tractatus_concept(self) -> str:
        return "What is agent delegation? Delegation = Task Analysis × Scope × Model × Agent × Resources"

    def _get_sequential_problem(self) -> str:
        return "How do we automatically determine optimal agent, model, and scope for tasks?"

    def _execute_analysis(self) -> Dict:
        # 1. Classify task
        task_class = self._classify_task()

        # 2. Detect scope
        scope = self._detect_scope(task_class)

        # 3. Select model
        model = self._select_model(task_class, scope)

        # 4. Route to agent
        agent = self._route_to_agent(task_class)

        return {
            'task_class': task_class,
            'scope': scope,
            'model': model,
            'agent': agent,
            'recommendation': self._generate_recommendation()
        }

    def _classify_task(self) -> Dict:
        # Analyze task description, files, dependencies
        # Return: type, complexity, language, requirements
        pass

    def _detect_scope(self, task_class: Dict) -> str:
        # Apply scope detection rules
        # Return: 'local', 'worktree', or 'cloud'
        pass

    def _select_model(self, task_class: Dict, scope: str) -> str:
        # Apply model selection rules
        # Return: 'auto', 'composer1', 'claude', etc.
        pass

    def _route_to_agent(self, task_class: Dict) -> str:
        # Apply agent routing rules
        # Return: 'backend', 'web', 'ipad', etc.
        pass
```

---

## Decision Matrix

| Task Type | Complexity | Files | Scope | Model | Agent |
|-----------|-----------|-------|-------|-------|-------|
| Quick fix | Low | 1-2 | Local | Auto | Backend/Web |
| Feature | Medium | 3-10 | Worktree | Composer1 | Specialized |
| Refactor | High | 10+ | Worktree | Composer1 | Backend |
| Analysis | Medium | N/A | Cloud | Claude | Data |
| Research | Low | N/A | Local | Claude | N/A |
| Architecture | High | Many | Worktree | Composer1 | Backend |

---

## Integration Points

### Todo2 Integration
- Add metadata fields: `scope`, `model`, `agent`
- Auto-populate on task creation
- Allow manual override
- Track delegation accuracy

### Cursor Integration
- Use Cursor's agent selection
- Leverage worktree support
- Integrate with model selection
- Provide delegation hints

### Learning System
- Track delegation success
- Learn from corrections
- Refine rules over time
- Improve accuracy

---

## Success Metrics

### Accuracy
- **Target**: 80%+ correct scope detection
- **Target**: 75%+ correct model selection
- **Target**: 85%+ correct agent routing

### Efficiency
- **Target**: Save 30% time on task setup
- **Measurement**: Time to start work before/after

### User Satisfaction
- **Target**: 90%+ tasks don't need manual override
- **Measurement**: Manual override rate

---

## Next Steps

1. **Research**: Understand Cursor's agent/model selection capabilities
2. **Prototype**: Build task classifier
3. **Test**: Validate scope detection rules
4. **Implement**: Create delegation automation
5. **Integrate**: Connect with Todo2 and Cursor

---

## Questions to Answer

1. **Cursor Capabilities**:
   - How does Cursor handle agent selection?
   - Can we programmatically set model/composer?
   - How does worktree detection work?

2. **Scope Detection**:
   - How do we detect if worktree is needed?
   - What triggers cloud execution?
   - How to detect local vs. isolated work?

3. **Model Selection**:
   - What models/composers are available?
   - How to determine optimal model?
   - Can we test model performance?

4. **Agent Routing**:
   - How are agents currently organized?
   - Can we programmatically route tasks?
   - How to handle agent availability?

---

*This automation will intelligently route tasks to optimal agents, models, and scopes for maximum efficiency.*
