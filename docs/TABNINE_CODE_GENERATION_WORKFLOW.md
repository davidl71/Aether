# Tabnine-Assisted Code Generation Workflow

**Date:** November 18, 2025
**Purpose:** Guide for using Tabnine to enhance code generation and development

---

## How Tabnine Works in This Workflow

### Understanding Tabnine's Role

**Tabnine is a passive enhancement tool:**

- ✅ Provides real-time code completions as you type
- ✅ Learns from your codebase patterns
- ✅ Suggests multi-line completions based on context
- ❌ Cannot be controlled programmatically
- ❌ Does not generate entire files automatically

### How I (AI Assistant) + Tabnine Work Together

1. **I Generate Initial Code Structure**
   - Create file skeletons with class definitions
   - Add method signatures and basic structure
   - Follow project patterns and conventions

2. **You Type/Edit Code**
   - Tabnine provides suggestions as you type
   - Tabnine learns from the codebase context
   - Tabnine suggests completions for methods, variables, patterns

3. **Iterative Enhancement**
   - I can review and improve generated code
   - Tabnine helps fill in implementation details
   - Together we create production-ready code

---

## Workflow for Plan Implementation

### Step 1: Code Structure Generation (AI Assistant)

I will generate:

- ✅ Header files with class definitions
- ✅ Method signatures matching the interface
- ✅ Basic implementation structure
- ✅ Error handling patterns
- ✅ Logging infrastructure

### Step 2: Tabnine-Assisted Implementation

As you type/edit:

- **Tabnine suggests:**
  - Variable names based on context
  - Method implementations following project patterns
  - Error handling code
  - Logging statements
  - Type conversions

### Step 3: Code Review & Refinement

- I review generated code
- Tabnine helps with refinements
- Together we ensure quality and consistency

---

## Current Plan Implementation Tasks

Based on `docs/PLAN_IMPLEMENTATION_SUMMARY.md`, these tasks need code generation:

### High Priority (T-35, T-36, T-37)

1. **T-35: Alpaca Adapter Implementation**
   - Generate: `native/include/brokers/alpaca_adapter.h`
   - Generate: `native/src/brokers/alpaca_adapter.cpp`
   - Tabnine will help with: HTTP client code, JSON parsing, error handling

2. **T-36: IB Client Portal Adapter Implementation**
   - Generate: `native/include/brokers/ib_client_portal_adapter.h`
   - Generate: `native/src/brokers/ib_client_portal_adapter.cpp`
   - Tabnine will help with: OAuth flow, REST API calls, session management

3. **T-37: Broker Selection Mechanism**
   - Generate: `native/include/brokers/broker_manager.h`
   - Generate: `native/src/brokers/broker_manager.cpp`
   - Tabnine will help with: Factory patterns, configuration management

### Prerequisites

First, I need to generate:

- ✅ `native/include/brokers/broker_interface.h` - Base interface
- ✅ `native/include/brokers/broker_types.h` - Common types

---

## Tabnine Configuration for This Workflow

### Optimal Settings

```json
{
  "tabnine.enableDeepCompletions": true,  // Multi-line suggestions
  "tabnine.enableInlinePredictions": true,  // Real-time suggestions
  "tabnine.maxNumResults": 5,  // Good balance
  "tabnine.debounceMs": 200  // Responsive but not overwhelming
}
```

### Language-Specific Benefits

**C++ Code:**

- Tabnine suggests: STL containers, smart pointers, RAII patterns
- Helps with: Type conversions, error handling, logging

**Python Code:**

- Tabnine suggests: Type hints, error handling, async patterns
- Helps with: API calls, data transformations

**TypeScript Code:**

- Tabnine suggests: React hooks, type definitions, async/await
- Helps with: API clients, state management

---

## Example: Tabnine-Assisted Development

### Scenario: Implementing Alpaca Adapter

**1. I Generate Structure:**

```cpp
class AlpacaAdapter : public brokers::IBroker {
public:
  bool connect() override;
  // ... more methods
};
```

**2. You Start Typing Implementation:**

```cpp
bool AlpacaAdapter::connect() {
  // Tabnine suggests: HTTP client initialization
  // Tabnine suggests: Authentication header setup
  // Tabnine suggests: Error handling pattern
}
```

**3. Tabnine Provides Context-Aware Suggestions:**

- Based on existing `TWSClient` patterns
- Following project logging conventions
- Matching error handling style

**4. I Review & Refine:**

- Ensure consistency with project patterns
- Add missing error cases
- Optimize performance

---

## Best Practices

### ✅ Do

1. **Let Tabnine Learn**
   - Type naturally, accept helpful suggestions
   - Tabnine learns from your codebase patterns

2. **Use Tabnine for Boilerplate**
   - Variable declarations
   - Error handling blocks
   - Logging statements
   - Type conversions

3. **Review Tabnine Suggestions**
   - Not all suggestions are perfect
   - Choose suggestions that match project style
   - Reject suggestions that don't fit

### ❌ Don't

1. **Don't Rely Solely on Tabnine**
   - Tabnine suggests, you decide
   - Always review generated code
   - Understand what code does

2. **Don't Accept Blindly**
   - Some suggestions may not match project patterns
   - Some may introduce security issues
   - Always review before accepting

---

## Next Steps

1. ✅ **Tabnine Installed** - Ready to use
2. 📝 **I Generate Code Structure** - Following plan
3. 🎯 **You Code with Tabnine** - Enhanced productivity
4. 🔍 **I Review & Refine** - Ensure quality

---

## Summary

**Tabnine enhances your coding experience:**

- Provides real-time suggestions as you type
- Learns from your codebase
- Speeds up boilerplate code
- Suggests patterns and best practices

**I generate the structure:**

- File skeletons
- Class definitions
- Method signatures
- Basic patterns

**Together we create:**

- Production-ready code
- Consistent patterns
- High-quality implementations

---

**Ready to start code generation!** 🚀
