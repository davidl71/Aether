#!/usr/bin/env python3
"""
Analyze TODO2 tasks to categorize interactive vs background execution modes.
"""

import json
import re
from collections import defaultdict

def analyze_tasks():
    with open('.todo2/state.todo2.json', 'r') as f:
        data = json.load(f)

    todos = data.get('todos', [])

    # Categorization
    interactive_tasks = []
    background_tasks = []
    ambiguous_tasks = []

    # Detailed categories
    interactive_categories = {
        'review_status': [],
        'needs_clarification': [],
        'design_decisions': [],
        'strategy_planning': [],
        'user_input_required': []
    }

    background_categories = {
        'implementation': [],
        'research': [],
        'testing': [],
        'documentation': [],
        'configuration': [],
        'refactoring': [],
        'mcp_extensions': []
    }

    for task in todos:
        task_id = task.get('id', '')
        name = task.get('name', '').lower()
        long_desc = task.get('long_description', '').lower()
        status = task.get('status', '')
        priority = task.get('priority', '')

        task_info = {
            'id': task_id,
            'name': task.get('name', ''),
            'status': status,
            'priority': priority,
            'has_exec_ctx': 'Best Mode' in task.get('long_description', ''),
            'reasons': []
        }

        # Strong interactive indicators
        is_review = status == 'Review'
        needs_clarification = 'clarification required' in long_desc or 'clarification' in long_desc
        needs_user_input = 'user input' in long_desc or 'user interaction' in long_desc or 'human approval' in long_desc
        is_design = 'design' in name and any(x in name for x in ['framework', 'system', 'strategy', 'allocation', 'investment'])
        is_decision = any(x in name for x in ['decide', 'choose', 'select', 'recommend', 'suggest', 'propose'])
        is_strategy = 'strategy' in name or 'strategy' in long_desc or 'plan' in name or 'workflow' in long_desc

        # Strong background indicators
        is_implementation = any(x in name for x in ['implement', 'create', 'add', 'update', 'fix', 'refactor'])
        is_research = 'research' in name
        is_testing = 'test' in name or 'testing' in name or 'validate' in name
        is_documentation = 'document' in name or 'documentation' in name
        is_configuration = 'config' in name or 'configure' in name or 'setup' in name
        is_mcp_extension = task_id.startswith('MCP-EXT')

        # Categorize
        is_interactive = is_review or needs_clarification or needs_user_input or (is_design and not is_implementation) or (is_decision and not is_implementation) or (is_strategy and not is_implementation)
        is_background = (is_implementation or is_research or is_testing or is_documentation or is_configuration or is_mcp_extension) and not is_interactive

        if is_interactive:
            interactive_tasks.append(task_info)

            if is_review:
                interactive_categories['review_status'].append(task_info)
                task_info['reasons'].append('Review status - needs human approval')
            if needs_clarification:
                interactive_categories['needs_clarification'].append(task_info)
                task_info['reasons'].append('Needs clarification')
            if needs_user_input:
                interactive_categories['user_input_required'].append(task_info)
                task_info['reasons'].append('Requires user input')
            if is_design:
                interactive_categories['design_decisions'].append(task_info)
                task_info['reasons'].append('Design decision required')
            if is_strategy:
                interactive_categories['strategy_planning'].append(task_info)
                task_info['reasons'].append('Strategy/Planning task')

        elif is_background:
            background_tasks.append(task_info)

            if is_mcp_extension:
                background_categories['mcp_extensions'].append(task_info)
                task_info['reasons'].append('MCP extension - autonomous')
            elif is_research:
                background_categories['research'].append(task_info)
                task_info['reasons'].append('Research - can run quietly')
            elif is_implementation:
                background_categories['implementation'].append(task_info)
                task_info['reasons'].append('Implementation - autonomous')
            elif is_testing:
                background_categories['testing'].append(task_info)
                task_info['reasons'].append('Testing - automated')
            elif is_documentation:
                background_categories['documentation'].append(task_info)
                task_info['reasons'].append('Documentation - autonomous')
            elif is_configuration:
                background_categories['configuration'].append(task_info)
                task_info['reasons'].append('Configuration - autonomous')
            else:
                background_categories['refactoring'].append(task_info)
                task_info['reasons'].append('Refactoring - autonomous')

        else:
            ambiguous_tasks.append({
                'id': task_id,
                'name': task.get('name', ''),
                'status': status
            })

    return {
        'interactive': interactive_tasks,
        'background': background_tasks,
        'ambiguous': ambiguous_tasks,
        'interactive_categories': interactive_categories,
        'background_categories': background_categories
    }

def print_analysis(results):
    print('=' * 70)
    print('TODO2 TASK EXECUTION MODE ANALYSIS')
    print('=' * 70)
    print()

    print(f'Total Tasks Analyzed: {len(results["interactive"]) + len(results["background"]) + len(results["ambiguous"])}')
    print()

    print('=' * 70)
    print('INTERACTIVE TASKS (Require User Input/Approval)')
    print('=' * 70)
    print(f'Total: {len(results["interactive"])} tasks\n')

    for category, tasks in results['interactive_categories'].items():
        if tasks:
            print(f'{category.upper().replace("_", " ")}: {len(tasks)} tasks')
            for task in tasks[:5]:
                reasons = ', '.join(task['reasons'][:2])
                exec_ctx = '✅' if task['has_exec_ctx'] else '❌'
                print(f'  {exec_ctx} {task["id"]}: {task["name"][:55]}')
                print(f'    Reason: {reasons} | Status: {task["status"]} | Priority: {task["priority"]}')
            if len(tasks) > 5:
                print(f'  ... and {len(tasks) - 5} more')
            print()

    print('=' * 70)
    print('BACKGROUND TASKS (Can Run Quietly)')
    print('=' * 70)
    print(f'Total: {len(results["background"])} tasks\n')

    for category, tasks in results['background_categories'].items():
        if tasks:
            print(f'{category.upper().replace("_", " ")}: {len(tasks)} tasks')
            for task in tasks[:5]:
                reasons = ', '.join(task['reasons'][:2])
                exec_ctx = '✅' if task['has_exec_ctx'] else '❌'
                print(f'  {exec_ctx} {task["id"]}: {task["name"][:55]}')
                print(f'    Reason: {reasons} | Status: {task["status"]} | Priority: {task["priority"]}')
            if len(tasks) > 5:
                print(f'  ... and {len(tasks) - 5} more')
            print()

    print('=' * 70)
    print('SUMMARY STATISTICS')
    print('=' * 70)
    print(f'Interactive Tasks: {len(results["interactive"])} ({len(results["interactive"])/(len(results["interactive"])+len(results["background"])+len(results["ambiguous"]))*100:.1f}%)')
    print(f'Background Tasks: {len(results["background"])} ({len(results["background"])/(len(results["interactive"])+len(results["background"])+len(results["ambiguous"]))*100:.1f}%)')
    print(f'Ambiguous Tasks: {len(results["ambiguous"])} ({len(results["ambiguous"])/(len(results["interactive"])+len(results["background"])+len(results["ambiguous"]))*100:.1f}%)')
    print()

    # High priority breakdown
    high_priority_interactive = [t for t in results['interactive'] if t['priority'] == 'high']
    high_priority_background = [t for t in results['background'] if t['priority'] == 'high']

    print('HIGH PRIORITY BREAKDOWN:')
    print(f'  Interactive (High Priority): {len(high_priority_interactive)} tasks')
    print(f'  Background (High Priority): {len(high_priority_background)} tasks')
    print()

    # Ready tasks (Todo status)
    ready_interactive = [t for t in results['interactive'] if t['status'] in ['Todo', 'todo']]
    ready_background = [t for t in results['background'] if t['status'] in ['Todo', 'todo']]

    print('READY TO START (Todo Status):')
    print(f'  Interactive Ready: {len(ready_interactive)} tasks')
    print(f'  Background Ready: {len(ready_background)} tasks')
    print()

if __name__ == '__main__':
    results = analyze_tasks()
    print_analysis(results)
