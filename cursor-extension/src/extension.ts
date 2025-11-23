import * as vscode from 'vscode';
import * as path from 'path';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

// MCP Tools and Prompts for autocomplete
const MCP_TOOLS = [
  { name: 'check_documentation_health_tool', description: 'Analyze documentation structure and health' },
  { name: 'analyze_todo2_alignment_tool', description: 'Analyze task alignment with project goals' },
  { name: 'detect_duplicate_tasks_tool', description: 'Detect and consolidate duplicate tasks' },
  { name: 'scan_dependency_security_tool', description: 'Scan dependencies for security vulnerabilities' },
  { name: 'find_automation_opportunities_tool', description: 'Discover automation opportunities' },
  { name: 'sync_todo_tasks_tool', description: 'Synchronize tasks across systems' },
  { name: 'review_pwa_config_tool', description: 'Review PWA configuration' },
  { name: 'server_status', description: 'Get MCP server status' }
];

const MCP_PROMPTS = [
  { name: 'doc_health_check', description: 'Documentation health check with task creation' },
  { name: 'doc_quick_check', description: 'Quick documentation check (no tasks)' },
  { name: 'task_alignment', description: 'Analyze task alignment with goals' },
  { name: 'duplicate_cleanup', description: 'Find and consolidate duplicate tasks' },
  { name: 'task_sync', description: 'Sync tasks between systems' },
  { name: 'security_scan_all', description: 'Scan all dependencies for vulnerabilities' },
  { name: 'security_scan_python', description: 'Scan Python dependencies' },
  { name: 'security_scan_rust', description: 'Scan Rust dependencies' },
  { name: 'automation_discovery', description: 'Discover automation opportunities' },
  { name: 'automation_high_value', description: 'Find high-value automation opportunities' },
  { name: 'pwa_review', description: 'Review PWA configuration' },
  { name: 'pre_sprint_cleanup', description: 'Pre-sprint cleanup workflow' },
  { name: 'post_implementation_review', description: 'Post-implementation review workflow' },
  { name: 'weekly_maintenance', description: 'Weekly maintenance workflow' }
];

interface MCPToolResult {
  success: boolean;
  data?: any;
  error?: string;
  timestamp?: string;
}

// Status bar items
let statusBarItem: vscode.StatusBarItem;
let serverStatusBarItem: vscode.StatusBarItem;
let lastOperationStatusBarItem: vscode.StatusBarItem;

export function activate(context: vscode.ExtensionContext) {
  console.log('Project Management Automation extension is now active!');

  // Get project root
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) {
    vscode.window.showWarningMessage('No workspace folder found. Project automation tools require a workspace.');
    return;
  }

  const projectRoot = workspaceFolder.uri.fsPath;
  const serverPath = path.join(projectRoot, 'mcp-servers', 'project-management-automation', 'run_server.sh');

  // Create status bar items
  statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
  statusBarItem.command = 'projectAutomation.showQuickActions';
  statusBarItem.tooltip = 'Project Automation - Click for quick actions';
  statusBarItem.text = '$(tools) Automation';
  statusBarItem.show();

  serverStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 99);
  serverStatusBarItem.command = 'projectAutomation.serverStatus';
  serverStatusBarItem.tooltip = 'MCP Server Status - Click to check';
  serverStatusBarItem.text = '$(sync~spin) Checking...';
  serverStatusBarItem.show();

  lastOperationStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 98);
  lastOperationStatusBarItem.tooltip = 'Last operation status';
  lastOperationStatusBarItem.text = '';
  // Don't show initially, only show after operations

  // Check server status on activation
  checkServerStatus();

  // Helper function to call MCP tool via Python
  async function callMCPTool(toolName: string, args: Record<string, any> = {}): Promise<MCPToolResult> {
    try {
      const venvPython = path.join(projectRoot, 'mcp-servers', 'project-management-automation', 'venv', 'bin', 'python3');
      const serverDir = path.join(projectRoot, 'mcp-servers', 'project-management-automation');

      // Map tool names to module/function names
      const toolMap: Record<string, { module: string; function: string }> = {
        'docs_health': { module: 'docs_health', function: 'check_documentation_health' },
        'todo2_alignment': { module: 'todo2_alignment', function: 'analyze_todo2_alignment' },
        'duplicate_detection': { module: 'duplicate_detection', function: 'detect_duplicate_tasks' },
        'dependency_security': { module: 'dependency_security', function: 'scan_dependency_security' },
        'automation_opportunities': { module: 'automation_opportunities', function: 'find_automation_opportunities' },
        'todo_sync': { module: 'todo_sync', function: 'sync_todo_tasks' },
        'pwa_review': { module: 'pwa_review', function: 'review_pwa_config' }
      };

      const toolInfo = toolMap[toolName];
      if (!toolInfo) {
        throw new Error(`Unknown tool: ${toolName}`);
      }

      // Build Python script to call the tool
      const argString = Object.entries(args)
        .map(([k, v]) => {
          if (v === null || v === undefined) {
            return `${k}=None`;
          } else if (typeof v === 'boolean') {
            return `${k}=${v}`;
          } else if (typeof v === 'number') {
            return `${k}=${v}`;
          } else if (Array.isArray(v)) {
            return `${k}=${JSON.stringify(v)}`;
          } else {
            return `${k}=${JSON.stringify(v)}`;
          }
        })
        .join(', ');

      const pythonScript = `
import sys
import json
import os
sys.path.insert(0, '${serverDir}')
sys.path.insert(0, '${projectRoot}')

os.chdir('${projectRoot}')

try:
    from tools.${toolInfo.module} import ${toolInfo.function}
    result = ${toolInfo.function}(${argString})
    print(json.dumps({"success": True, "data": json.loads(result), "timestamp": __import__('datetime').datetime.now().isoformat()}))
except Exception as e:
    print(json.dumps({"success": False, "error": str(e), "timestamp": __import__('datetime').datetime.now().isoformat()}))
    sys.exit(1)
`;

      const { stdout, stderr } = await execAsync(
        `"${venvPython}" -c ${JSON.stringify(pythonScript)}`,
        {
          cwd: projectRoot,
          maxBuffer: 10 * 1024 * 1024 // 10MB buffer
        }
      );

      if (stderr && !stderr.includes('INFO') && !stderr.includes('WARNING')) {
        console.error('MCP Tool Error:', stderr);
      }

      const result = JSON.parse(stdout.trim());
      return result;
    } catch (error: any) {
      console.error('Error calling MCP tool:', error);
      return {
        success: false,
        error: error.message || 'Unknown error',
        timestamp: new Date().toISOString()
      };
    }
  }

  // Documentation Health Check
  const docHealthCmd = vscode.commands.registerCommand('projectAutomation.documentationHealth', async () => {
    updateStatusBar('running');
    const createTasks = await vscode.window.showQuickPick(
      ['Yes, create tasks', 'No, report only'],
      { placeHolder: 'Create Todo2 tasks for issues?' }
    );

    if (!createTasks) {
      updateStatusBar('idle');
      return;
    }

    const output = vscode.window.createOutputChannel('Documentation Health');
    output.show();
    output.appendLine('Checking documentation health...');

    try {
      const result = await callMCPTool('docs_health', {
        create_tasks: createTasks === 'Yes, create tasks',
        output_path: path.join(projectRoot, 'docs', 'DOCUMENTATION_HEALTH_REPORT.md')
      });

      if (result.success) {
        const score = result.data?.health_score || 'N/A';
        output.appendLine(`✅ Documentation Health Check Complete`);
        output.appendLine(`Health Score: ${score}`);
        output.appendLine(`Report: ${result.data?.report_path || 'N/A'}`);
        updateStatusBar('success', `Docs: ${score}`);
        vscode.window.showInformationMessage(`Documentation health check completed! Score: ${score}`);
      } else {
        output.appendLine(`❌ Error: ${result.error}`);
        updateStatusBar('error', 'Docs check failed');
        vscode.window.showErrorMessage(`Documentation health check failed: ${result.error}`);
      }
    } catch (error: any) {
      output.appendLine(`❌ Error: ${error.message}`);
      updateStatusBar('error', 'Docs check error');
      vscode.window.showErrorMessage(`Error: ${error.message}`);
    }
  });

  // Task Alignment Analysis
  const taskAlignmentCmd = vscode.commands.registerCommand('projectAutomation.taskAlignment', async () => {
    updateStatusBar('running');
    const output = vscode.window.createOutputChannel('Task Alignment');
    output.show();
    output.appendLine('Analyzing task alignment...');

    try {
      const result = await callMCPTool('todo2_alignment', {
        create_followup_tasks: true,
        output_path: path.join(projectRoot, 'docs', 'TODO2_ALIGNMENT_REPORT.md')
      });

      if (result.success) {
        const misaligned = result.data?.misaligned_count || 0;
        output.appendLine(`✅ Task Alignment Analysis Complete`);
        output.appendLine(`Total Tasks: ${result.data?.total_tasks || 'N/A'}`);
        output.appendLine(`Misaligned: ${misaligned}`);
        updateStatusBar('success', `Tasks: ${misaligned} misaligned`);
        vscode.window.showInformationMessage(`Task alignment analysis completed! ${misaligned} misaligned tasks found.`);
      } else {
        output.appendLine(`❌ Error: ${result.error}`);
        updateStatusBar('error', 'Alignment failed');
        vscode.window.showErrorMessage(`Task alignment failed: ${result.error}`);
      }
    } catch (error: any) {
      output.appendLine(`❌ Error: ${error.message}`);
      updateStatusBar('error', 'Alignment error');
      vscode.window.showErrorMessage(`Error: ${error.message}`);
    }
  });

  // Duplicate Task Detection
  const duplicateTasksCmd = vscode.commands.registerCommand('projectAutomation.duplicateTasks', async () => {
    const threshold = await vscode.window.showInputBox({
      prompt: 'Similarity threshold (0.0-1.0)',
      value: '0.85',
      validateInput: (value) => {
        const num = parseFloat(value);
        if (isNaN(num) || num < 0 || num > 1) {
          return 'Please enter a number between 0.0 and 1.0';
        }
        return null;
      }
    });

    if (!threshold) { return; }

    const autoFix = await vscode.window.showQuickPick(['Yes', 'No'], {
      placeHolder: 'Auto-fix duplicates?'
    });

    const output = vscode.window.createOutputChannel('Duplicate Detection');
    output.show();
    output.appendLine('Detecting duplicate tasks...');

    try {
      const result = await callMCPTool('duplicate_detection', {
        similarity_threshold: parseFloat(threshold),
        auto_fix: autoFix === 'Yes',
        output_path: path.join(projectRoot, 'docs', 'DUPLICATE_TASKS_REPORT.md')
      });

      if (result.success) {
        output.appendLine(`✅ Duplicate Detection Complete`);
        output.appendLine(`Duplicates Found: ${result.data?.total_duplicates || 'N/A'}`);
        vscode.window.showInformationMessage('Duplicate detection completed!');
      } else {
        output.appendLine(`❌ Error: ${result.error}`);
        vscode.window.showErrorMessage(`Duplicate detection failed: ${result.error}`);
      }
    } catch (error: any) {
      output.appendLine(`❌ Error: ${error.message}`);
      vscode.window.showErrorMessage(`Error: ${error.message}`);
    }
  });

  // Security Scan
  const securityScanCmd = vscode.commands.registerCommand('projectAutomation.securityScan', async () => {
    const languages = await vscode.window.showQuickPick(
      ['All Languages', 'Python', 'Rust', 'npm'],
      { placeHolder: 'Select languages to scan', canPickMany: true }
    );

    if (!languages) { return; }

    const langList = languages.includes('All Languages') ? null :
      languages.map(l => l.toLowerCase().replace('npm', 'npm'));

    const output = vscode.window.createOutputChannel('Security Scan');
    output.show();
    output.appendLine('Scanning dependencies for security vulnerabilities...');

    try {
      const result = await callMCPTool('dependency_security', {
        languages: langList
      });

      if (result.success) {
        output.appendLine(`✅ Security Scan Complete`);
        output.appendLine(`Total Vulnerabilities: ${result.data?.total_vulnerabilities || 'N/A'}`);
        output.appendLine(`Critical: ${result.data?.critical_count || 'N/A'}`);
        vscode.window.showInformationMessage('Security scan completed!');
      } else {
        output.appendLine(`❌ Error: ${result.error}`);
        vscode.window.showErrorMessage(`Security scan failed: ${result.error}`);
      }
    } catch (error: any) {
      output.appendLine(`❌ Error: ${error.message}`);
      vscode.window.showErrorMessage(`Error: ${error.message}`);
    }
  });

  // Automation Discovery
  const automationDiscoveryCmd = vscode.commands.registerCommand('projectAutomation.automationDiscovery', async () => {
    const minScore = await vscode.window.showInputBox({
      prompt: 'Minimum value score (0.0-1.0)',
      value: '0.7',
      validateInput: (value) => {
        const num = parseFloat(value);
        if (isNaN(num) || num < 0 || num > 1) {
          return 'Please enter a number between 0.0 and 1.0';
        }
        return null;
      }
    });

    if (!minScore) { return; }

    const output = vscode.window.createOutputChannel('Automation Discovery');
    output.show();
    output.appendLine('Discovering automation opportunities...');

    try {
      const result = await callMCPTool('automation_opportunities', {
        min_value_score: parseFloat(minScore),
        output_path: path.join(projectRoot, 'docs', 'AUTOMATION_OPPORTUNITIES.md')
      });

      if (result.success) {
        output.appendLine(`✅ Automation Discovery Complete`);
        output.appendLine(`Opportunities Found: ${result.data?.total_opportunities || 'N/A'}`);
        vscode.window.showInformationMessage('Automation discovery completed!');
      } else {
        output.appendLine(`❌ Error: ${result.error}`);
        vscode.window.showErrorMessage(`Automation discovery failed: ${result.error}`);
      }
    } catch (error: any) {
      output.appendLine(`❌ Error: ${error.message}`);
      vscode.window.showErrorMessage(`Error: ${error.message}`);
    }
  });

  // Sync Tasks
  const syncTasksCmd = vscode.commands.registerCommand('projectAutomation.syncTasks', async () => {
    updateStatusBar('running');
    const dryRun = await vscode.window.showQuickPick(['Yes (preview)', 'No (apply changes)'], {
      placeHolder: 'Dry run mode?'
    });

    if (!dryRun) {
      updateStatusBar('idle');
      return;
    }

    const output = vscode.window.createOutputChannel('Task Sync');
    output.show();
    output.appendLine('Synchronizing tasks across systems...');

    try {
      const result = await callMCPTool('todo_sync', {
        dry_run: dryRun === 'Yes (preview)',
        output_path: path.join(projectRoot, 'docs', 'TODO_SYNC_REPORT.md')
      });

      if (result.success) {
        const matches = result.data?.matches_found || 0;
        const conflicts = result.data?.conflicts_detected || 0;
        const newTasks = result.data?.new_todo2_tasks || 0;
        output.appendLine(`✅ Task Sync Complete`);
        output.appendLine(`Matches: ${matches}`);
        output.appendLine(`Conflicts: ${conflicts}`);
        output.appendLine(`New Tasks: ${newTasks}`);
        updateStatusBar('success', `Sync: ${matches} matches, ${newTasks} new`);
        vscode.window.showInformationMessage(`Task synchronization completed! ${matches} matches, ${newTasks} new tasks.`);
      } else {
        output.appendLine(`❌ Error: ${result.error}`);
        updateStatusBar('error', 'Sync failed');
        vscode.window.showErrorMessage(`Task sync failed: ${result.error}`);
      }
    } catch (error: any) {
      output.appendLine(`❌ Error: ${error.message}`);
      updateStatusBar('error', 'Sync error');
      vscode.window.showErrorMessage(`Error: ${error.message}`);
    }
  });

  // PWA Review
  const pwaReviewCmd = vscode.commands.registerCommand('projectAutomation.pwaReview', async () => {
    const output = vscode.window.createOutputChannel('PWA Review');
    output.show();
    output.appendLine('Reviewing PWA configuration...');

    try {
      const result = await callMCPTool('pwa_review', {
        output_path: path.join(projectRoot, 'docs', 'PWA_REVIEW_REPORT.md')
      });

      if (result.success) {
        output.appendLine(`✅ PWA Review Complete`);
        output.appendLine(`Report: ${result.data?.report_path || 'N/A'}`);
        vscode.window.showInformationMessage('PWA review completed!');
      } else {
        output.appendLine(`❌ Error: ${result.error}`);
        vscode.window.showErrorMessage(`PWA review failed: ${result.error}`);
      }
    } catch (error: any) {
      output.appendLine(`❌ Error: ${error.message}`);
      vscode.window.showErrorMessage(`Error: ${error.message}`);
    }
  });

  // Pre-Sprint Cleanup Workflow
  const preSprintCleanupCmd = vscode.commands.registerCommand('projectAutomation.preSprintCleanup', async () => {
    updateStatusBar('running');
    const output = vscode.window.createOutputChannel('Pre-Sprint Cleanup');
    output.show();
    output.appendLine('Running pre-sprint cleanup workflow...');
    output.appendLine('1. Detecting duplicate tasks...');

    // Step 1: Duplicate detection
    const dupResult = await callMCPTool('duplicate_detection', {
      similarity_threshold: 0.85,
      auto_fix: false
    });
    output.appendLine(`   ${dupResult.success ? '✅' : '❌'} Duplicate detection: ${dupResult.data?.total_duplicates || 0} found`);

    output.appendLine('2. Analyzing task alignment...');
    const alignResult = await callMCPTool('todo2_alignment', {
      create_followup_tasks: true
    });
    output.appendLine(`   ${alignResult.success ? '✅' : '❌'} Task alignment: ${alignResult.data?.misaligned_count || 0} misaligned`);

    output.appendLine('3. Checking documentation health...');
    const docResult = await callMCPTool('docs_health', {
      create_tasks: true
    });
    output.appendLine(`   ${docResult.success ? '✅' : '❌'} Documentation: Score ${docResult.data?.health_score || 'N/A'}`);

    output.appendLine('\n✅ Pre-sprint cleanup workflow complete!');
    updateStatusBar('success', 'Pre-sprint complete');
    vscode.window.showInformationMessage('Pre-sprint cleanup workflow completed!');
  });

  // Post-Implementation Review Workflow
  const postImplementationReviewCmd = vscode.commands.registerCommand('projectAutomation.postImplementationReview', async () => {
    const output = vscode.window.createOutputChannel('Post-Implementation Review');
    output.show();
    output.appendLine('Running post-implementation review workflow...');

    output.appendLine('1. Checking documentation health...');
    const docResult = await callMCPTool('docs_health', { create_tasks: true });
    output.appendLine(`   ${docResult.success ? '✅' : '❌'} Documentation updated`);

    output.appendLine('2. Scanning for security vulnerabilities...');
    const secResult = await callMCPTool('dependency_security', { languages: null });
    output.appendLine(`   ${secResult.success ? '✅' : '❌'} Security scan: ${secResult.data?.total_vulnerabilities || 0} vulnerabilities`);

    output.appendLine('3. Discovering automation opportunities...');
    const autoResult = await callMCPTool('automation_opportunities', { min_value_score: 0.7 });
    output.appendLine(`   ${autoResult.success ? '✅' : '❌'} Automation: ${autoResult.data?.total_opportunities || 0} opportunities`);

    output.appendLine('\n✅ Post-implementation review complete!');
    vscode.window.showInformationMessage('Post-implementation review completed!');
  });

  // Weekly Maintenance Workflow
  const weeklyMaintenanceCmd = vscode.commands.registerCommand('projectAutomation.weeklyMaintenance', async () => {
    updateStatusBar('running');
    const output = vscode.window.createOutputChannel('Weekly Maintenance');
    output.show();
    output.appendLine('Running weekly maintenance workflow...');

    output.appendLine('1. Checking documentation health...');
    const docResult = await callMCPTool('docs_health', { create_tasks: true });
    output.appendLine(`   ${docResult.success ? '✅' : '❌'} Documentation: Score ${docResult.data?.health_score || 'N/A'}`);

    output.appendLine('2. Cleaning up duplicate tasks...');
    const dupResult = await callMCPTool('duplicate_detection', {
      similarity_threshold: 0.85,
      auto_fix: false
    });
    output.appendLine(`   ${dupResult.success ? '✅' : '❌'} Duplicates: ${dupResult.data?.total_duplicates || 0} found`);

    output.appendLine('3. Scanning dependencies for security...');
    const secResult = await callMCPTool('dependency_security', { languages: null });
    output.appendLine(`   ${secResult.success ? '✅' : '❌'} Security: ${secResult.data?.total_vulnerabilities || 0} vulnerabilities`);

    output.appendLine('4. Synchronizing tasks...');
    const syncResult = await callMCPTool('todo_sync', { dry_run: false });
    output.appendLine(`   ${syncResult.success ? '✅' : '❌'} Sync: ${syncResult.data?.matches_found || 0} matches`);

    output.appendLine('\n✅ Weekly maintenance complete!');
    updateStatusBar('success', 'Maintenance complete');
    vscode.window.showInformationMessage('Weekly maintenance completed!');
  });

  // Helper function to update status bar
  function updateStatusBar(status: 'idle' | 'running' | 'success' | 'error', message?: string) {
    if (status === 'running') {
      statusBarItem.text = '$(sync~spin) Running...';
      statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.prominentForeground');
    } else if (status === 'success') {
      statusBarItem.text = '$(check) Automation';
      statusBarItem.backgroundColor = undefined;
      if (message) {
        lastOperationStatusBarItem.text = `$(check) ${message}`;
        lastOperationStatusBarItem.backgroundColor = undefined;
        lastOperationStatusBarItem.show();
        setTimeout(() => {
          lastOperationStatusBarItem.hide();
        }, 3000);
      }
    } else if (status === 'error') {
      statusBarItem.text = '$(error) Automation';
      statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
      if (message) {
        lastOperationStatusBarItem.text = `$(error) ${message}`;
        lastOperationStatusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
        lastOperationStatusBarItem.show();
        setTimeout(() => {
          lastOperationStatusBarItem.hide();
        }, 5000);
      }
    } else {
      statusBarItem.text = '$(tools) Automation';
      statusBarItem.backgroundColor = undefined;
    }
  }

  // Check server status
  async function checkServerStatus() {
    try {
      const fs = require('fs');
      const venvPython = path.join(projectRoot, 'mcp-servers', 'project-management-automation', 'venv', 'bin', 'python3');
      const serverScript = path.join(projectRoot, 'mcp-servers', 'project-management-automation', 'run_server.sh');

      if (fs.existsSync(serverScript) && fs.existsSync(venvPython)) {
        serverStatusBarItem.text = '$(check) Server Ready';
        serverStatusBarItem.backgroundColor = undefined;
        serverStatusBarItem.tooltip = 'MCP Server: Operational - Click to check details';
      } else {
        serverStatusBarItem.text = '$(warning) Server Not Found';
        serverStatusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
        serverStatusBarItem.tooltip = 'MCP Server: Not found - Check configuration';
      }
    } catch (error) {
      serverStatusBarItem.text = '$(error) Server Error';
      serverStatusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
      serverStatusBarItem.tooltip = 'MCP Server: Error checking status';
    }
  }

  // Quick Actions Menu
  const quickActionsCmd = vscode.commands.registerCommand('projectAutomation.showQuickActions', async () => {
    const items = [
      { label: '$(file-text) Documentation Health', description: 'Check documentation health', command: 'projectAutomation.documentationHealth' },
      { label: '$(list-unordered) Task Alignment', description: 'Analyze task alignment', command: 'projectAutomation.taskAlignment' },
      { label: '$(duplicate) Duplicate Tasks', description: 'Detect duplicate tasks', command: 'projectAutomation.duplicateTasks' },
      { label: '$(shield) Security Scan', description: 'Scan dependencies', command: 'projectAutomation.securityScan' },
      { label: '$(sync) Sync Tasks', description: 'Sync across systems', command: 'projectAutomation.syncTasks' },
      { label: '$(rocket) Pre-Sprint Cleanup', description: 'Run pre-sprint workflow', command: 'projectAutomation.preSprintCleanup' },
      { label: '$(checklist) Weekly Maintenance', description: 'Run weekly maintenance', command: 'projectAutomation.weeklyMaintenance' },
      { label: '$(info) Server Status', description: 'Show server status', command: 'projectAutomation.serverStatus' }
    ];

    const selected = await vscode.window.showQuickPick(items, {
      placeHolder: 'Select an automation tool or workflow...'
    });

    if (selected) {
      vscode.commands.executeCommand(selected.command);
    }
  });

  // Server Status
  const serverStatusCmd = vscode.commands.registerCommand('projectAutomation.serverStatus', async () => {
    const output = vscode.window.createOutputChannel('Server Status');
    output.show();
    output.appendLine('Checking server status...');

    try {
      // Check if server script exists
      const fs = require('fs');
      if (!fs.existsSync(serverPath)) {
        output.appendLine('❌ Server script not found');
        vscode.window.showErrorMessage('MCP server script not found');
        return;
      }

      output.appendLine('✅ Server script found');
      output.appendLine(`Path: ${serverPath}`);
      output.appendLine('\nAvailable Tools:');
      output.appendLine('  - Documentation Health Check');
      output.appendLine('  - Task Alignment Analysis');
      output.appendLine('  - Duplicate Task Detection');
      output.appendLine('  - Security Scanning');
      output.appendLine('  - Automation Discovery');
      output.appendLine('  - Task Synchronization');
      output.appendLine('  - PWA Review');
      output.appendLine('\nWorkflows:');
      output.appendLine('  - Pre-Sprint Cleanup');
      output.appendLine('  - Post-Implementation Review');
      output.appendLine('  - Weekly Maintenance');

      // Update status bar
      await checkServerStatus();
      vscode.window.showInformationMessage('Server status: Operational');
    } catch (error: any) {
      output.appendLine(`❌ Error: ${error.message}`);
      vscode.window.showErrorMessage(`Error: ${error.message}`);
    }
  });

  // MCP Tools/Prompts Completion Provider
  const completionProvider = vscode.languages.registerCompletionItemProvider(
    [
      { scheme: 'file', language: 'markdown' },
      { scheme: 'file', language: 'plaintext' }
    ],
    {
      provideCompletionItems(document: vscode.TextDocument, position: vscode.Position) {
        const linePrefix = document.lineAt(position).text.substr(0, position.character);
        const completions: vscode.CompletionItem[] = [];

        // Suggest MCP tools when typing "Use " or "use " or "check_" or "analyze_" etc.
        if (linePrefix.match(/\b(Use|use|Run|run|Call|call|Execute|execute)\s+.*$/i) ||
            linePrefix.match(/\b(check_|analyze_|detect_|scan_|find_|sync_|review_)/i)) {

          MCP_TOOLS.forEach(tool => {
            const item = new vscode.CompletionItem(
              tool.name,
              vscode.CompletionItemKind.Function
            );
            item.detail = `MCP Tool: ${tool.description}`;
            item.documentation = new vscode.MarkdownString(`**MCP Tool:** ${tool.name}\n\n${tool.description}\n\nUse this tool in chat to automate project management tasks.`);
            item.insertText = new vscode.SnippetString(`${tool.name}($0)`);
            completions.push(item);
          });
        }

        // Suggest MCP prompts when typing "prompt" or "Prompt" or prompt names
        if (linePrefix.match(/\b(prompt|Prompt|Use the|use the)\s+.*$/i) ||
            linePrefix.match(/\b(doc_|task_|duplicate_|security_|automation_|pwa_|pre_|post_|weekly_)/i)) {

          MCP_PROMPTS.forEach(prompt => {
            const item = new vscode.CompletionItem(
              prompt.name,
              vscode.CompletionItemKind.Snippet
            );
            item.detail = `MCP Prompt: ${prompt.description}`;
            item.documentation = new vscode.MarkdownString(`**MCP Prompt:** ${prompt.name}\n\n${prompt.description}\n\nUse this prompt in chat: "Use the ${prompt.name} prompt"`);
            item.insertText = new vscode.SnippetString(`Use the ${prompt.name} prompt`);
            completions.push(item);
          });
        }

        return completions;
      }
    },
    '.', '_' // Trigger on . and _
  );

  // Command to show available tools/prompts
  const showMCPHelpCmd = vscode.commands.registerCommand('projectAutomation.showMCPHelp', async () => {
    const output = vscode.window.createOutputChannel('MCP Tools & Prompts');
    output.show();

    output.appendLine('📋 Available MCP Tools');
    output.appendLine('='.repeat(50));
    MCP_TOOLS.forEach(tool => {
      output.appendLine(`\n🔧 ${tool.name}`);
      output.appendLine(`   ${tool.description}`);
      output.appendLine(`   Usage in chat: "Use ${tool.name}"`);
    });

    output.appendLine('\n\n📝 Available MCP Prompts');
    output.appendLine('='.repeat(50));
    MCP_PROMPTS.forEach(prompt => {
      output.appendLine(`\n💡 ${prompt.name}`);
      output.appendLine(`   ${prompt.description}`);
      output.appendLine(`   Usage in chat: "Use the ${prompt.name} prompt"`);
    });

    output.appendLine('\n\n💡 Autocomplete Tips:');
    output.appendLine('   - Type "Use " or "use " in chat to see tool suggestions');
    output.appendLine('   - Type "prompt" or prompt names to see prompt suggestions');
    output.appendLine('   - Use snippets: Type "mcp-" prefix for quick insertion');
    output.appendLine('   - Command Palette: "Project Automation: Show MCP Help"');

    vscode.window.showInformationMessage('MCP Tools & Prompts help opened in output channel');
  });

  // Register all commands
  context.subscriptions.push(
    statusBarItem,
    serverStatusBarItem,
    lastOperationStatusBarItem,
    quickActionsCmd,
    docHealthCmd,
    taskAlignmentCmd,
    duplicateTasksCmd,
    securityScanCmd,
    automationDiscoveryCmd,
    syncTasksCmd,
    pwaReviewCmd,
    preSprintCleanupCmd,
    postImplementationReviewCmd,
    weeklyMaintenanceCmd,
    serverStatusCmd,
    showMCPHelpCmd,
    completionProvider
  );
}

export function deactivate() {}
