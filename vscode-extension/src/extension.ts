import * as vscode from 'vscode';
import { exec } from 'child_process';

// Advanced debounce: global queue system to prevent duplicate analysis
const analysisQueue = new Map<string, {
    timer: NodeJS.Timeout;
    document: vscode.TextDocument;
    requestId: number;
}>();
const activeAnalysis = new Set<string>();
let globalRequestId = 0;
const languageCache = new Map<string, string>();  // Cache for detected languages
const DEBOUNCE_MS = 800;

/**
 * Escape shell special characters to prevent command injection
 */
function escapeShellArg(arg: string): string {
    return `'${arg.replace(/'/g, "'\\''")}'`;
}

// Issue interface matching Rust CodeIssue struct
interface GarbageIssue {
    file_path: string;
    line: number;
    column: number;
    rule_name: string;
    message: string;
    severity: 'Mild' | 'Spicy' | 'Nuclear';
}

// Global diagnostic collection
let diagnosticCollection: vscode.DiagnosticCollection;

// Output channel for educational mode
let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
    console.log('🗑️ Garbage Code Hunter is now active!');

    // Create diagnostic collection (this works correctly, no duplicates!)
    diagnosticCollection = vscode.languages.createDiagnosticCollection('garbage-hunter');

    // Create output channel for educational/score reports
    outputChannel = vscode.window.createOutputChannel('Garbage Hunter');

    // Check CLI availability on startup
    checkCliAvailability();

    // Register commands
    registerCommands(context);

    // Register file watchers
    registerFileWatchers(context);

    // Register configuration watcher
    registerConfigurationWatcher(context);

    // Delay initial analysis to avoid race conditions with other event listeners
    setTimeout(() => {
        analyzeOpenRustFiles();
    }, 1000);

    context.subscriptions.push(diagnosticCollection, outputChannel);
}

function getCliCommand(): string {
    const config = vscode.workspace.getConfiguration('garbageHunter');
    const cliPath = config.get<string>('cliPath', '');
    return cliPath || 'garbage-code-hunter';
}

async function checkCliAvailability() {
    const cli = getCliCommand();
    exec(`${cli} --version`, (error, stdout, stderr) => {
        if (error) {
            const msg = error.message || '';
            if (msg.includes('ENOENT') || msg.includes('not found')) {
                vscode.window.showWarningMessage(
                    '🗑️ Garbage Code Hunter: CLI not found. Install it with `cargo install garbage-code-hunter` or set `garbageHunter.cliPath` in settings.',
                    'Open Settings',
                    'Copy Install Command'
                ).then(selection => {
                    if (selection === 'Open Settings') {
                        vscode.commands.executeCommand('workbench.action.openSettings', 'garbageHunter.cliPath');
                    } else if (selection === 'Copy Install Command') {
                        vscode.env.clipboard.writeText('cargo install garbage-code-hunter');
                    }
                });
            }
        } else {
            console.log(`🗑️ Garbage Code Hunter CLI found: ${stdout.trim()}`);
        }
    });
}

function registerCommands(context: vscode.ExtensionContext) {
    // Analyze current file
    const analyzeFileCommand = vscode.commands.registerCommand(
        'garbageHunter.analyzeFile',
        () => analyzeCurrentFile()
    );

    // Analyze entire workspace
    const analyzeWorkspaceCommand = vscode.commands.registerCommand(
        'garbageHunter.analyzeWorkspace',
        () => analyzeWorkspace()
    );

    // Clear all diagnostics
    const clearDiagnosticsCommand = vscode.commands.registerCommand(
        'garbageHunter.clearDiagnostics',
        () => {
            diagnosticCollection.clear();
            vscode.window.showInformationMessage('🧹 All roasts cleared!');
        }
    );

    // Show quality score summary
    const showScoreCommand = vscode.commands.registerCommand(
        'garbageHunter.showScore',
        () => showQualityScore()
    );

    // Show educational advice
    const showEducationalCommand = vscode.commands.registerCommand(
        'garbageHunter.showEducational',
        () => showEducationalAdvice()
    );

    context.subscriptions.push(
        analyzeFileCommand,
        analyzeWorkspaceCommand,
        clearDiagnosticsCommand,
        showScoreCommand,
        showEducationalCommand
    );
}

function registerFileWatchers(context: vscode.ExtensionContext) {
    // Listen for file saves
    const onSaveListener = vscode.workspace.onDidSaveTextDocument((document) => {
        const config = vscode.workspace.getConfiguration('garbageHunter');
        if (config.get('enableRealTimeAnalysis', true) && document.languageId === 'rust') {
            analyzeDocument(document);
        }
    });

    context.subscriptions.push(onSaveListener);
}

function registerConfigurationWatcher(context: vscode.ExtensionContext) {
    const configWatcher = vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration('garbageHunter')) {
            // Recreate decoration types on config change

            // Re-analyze all open files
            analyzeOpenRustFiles();
        }
    });

    context.subscriptions.push(configWatcher);
}

async function analyzeCurrentFile() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showWarningMessage('No active file to analyze');
        return;
    }

    if (editor.document.languageId !== 'rust') {
        vscode.window.showWarningMessage('This is not a Rust file');
        return;
    }

    await analyzeDocument(editor.document);
}

async function analyzeWorkspace() {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) {
        vscode.window.showWarningMessage('No workspace folder found');
        return;
    }

    await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: "🔥 Roasting your entire codebase...",
        cancellable: false
    }, async (progress) => {
        progress.report({ increment: 0, message: "Starting analysis..." });

        try {
            const issues = await runGarbageHunterOnPath(workspaceFolder.uri.fsPath);

            // Clear previous diagnostics
            diagnosticCollection.clear();

            // Group issues by file
            const issuesByFile = groupIssuesByFile(issues);

            for (const [filePath, fileIssues] of issuesByFile) {
                const uri = vscode.Uri.file(filePath);
                const diagnostics = issuesToDiagnostics(fileIssues);
                diagnosticCollection.set(uri, diagnostics);
            }

            progress.report({ increment: 100, message: "Analysis complete!" });

            const totalIssues = issues.length;
            const fileCount = issuesByFile.size;

            if (totalIssues === 0) {
                vscode.window.showInformationMessage('🎉 Your codebase is surprisingly clean!');
            } else {
                vscode.window.showWarningMessage(
                    `🗑️ Found ${totalIssues} garbage issues across ${fileCount} files`,
                    'Show Problems'
                ).then(selection => {
                    if (selection === 'Show Problems') {
                        vscode.commands.executeCommand('workbench.panel.markers.view.focus');
                    }
                });
            }
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);
            vscode.window.showErrorMessage(`🗑️ Analysis failed: ${errorMessage}`);
            console.error('Workspace analysis error:', error);
        }
    });
}

async function analyzeDocument(document: vscode.TextDocument) {
    if (shouldExcludeFile(document.uri.fsPath)) {
        return;
    }

    const filePath = document.uri.fsPath;

    // Generate unique request ID for this analysis
    const currentRequestId = ++globalRequestId;

    console.log(`📨 [QUEUE] Request #${currentRequestId} for: ${filePath}`);
    console.log(`📨 [QUEUE] Active analyses: ${Array.from(activeAnalysis).join(', ') || 'none'}`);

    // Cancel any existing queued analysis for this file
    const existing = analysisQueue.get(filePath);
    if (existing) {
        console.log(`❌ [QUEUE] Cancelling request #${existing.requestId} (replaced by #${currentRequestId})`);
        clearTimeout(existing.timer);
        analysisQueue.delete(filePath);
    }

    // If already actively analyzing this file, queue a new analysis after completion
    if (activeAnalysis.has(filePath)) {
        console.log(`⏳ [QUEUE] File currently being analyzed, queuing request #${currentRequestId}`);
        const timer = setTimeout(() => {
            analysisQueue.delete(filePath);
            console.log(`⏰ [QUEUE] Executing queued request #${currentRequestId}`);
            analyzeDocument(document);
        }, DEBOUNCE_MS * 2);  // Wait longer for queued requests

        analysisQueue.set(filePath, { timer, document, requestId: currentRequestId });
        return;
    }

    // Debounce: wait before actually analyzing
    console.log(`⏱️ [QUEUE] Scheduling request #${currentRequestId} with ${DEBOUNCE_MS}ms debounce`);
    const timer = setTimeout(async () => {
        analysisQueue.delete(filePath);
        await executeAnalysis(document, currentRequestId);
    }, DEBOUNCE_MS);

    analysisQueue.set(filePath, { timer, document, requestId: currentRequestId });
}

async function executeAnalysis(document: vscode.TextDocument, requestId: number) {
    const filePath = document.uri.fsPath;

    // Final safety check
    if (activeAnalysis.has(filePath)) {
        console.log(`� [EXEC] Request #${requestId} cancelled - file already being analyzed`);
        return;
    }

    console.log(`🚀 [EXEC] Starting request #${requestId} for: ${filePath}`);
    console.log(`🚀 [EXEC] Timestamp: ${new Date().toISOString()}`);
    activeAnalysis.add(filePath);

    try {
        const issues = await runGarbageHunterOnPath(document.fileName);
        console.log(`📥 [EXEC] Request #${requestId}: CLI returned ${issues.length} issues`);

        // CRITICAL: Check for duplicates in CLI output BEFORE processing
        if (issues.length > 0) {
            const issueKeys = issues.map(i => `${i.line}|${i.rule_name}|${i.message}`);
            const uniqueKeys = new Set(issueKeys);

            if (issueKeys.length !== uniqueKeys.size) {
                console.log(`🚨 [EXEC] CLI RETURNED DUPLICATES! Total: ${issueKeys.length}, Unique: ${uniqueKeys.size}`);
                console.log(`🚨 [EXEC] This means your Rust CLI tool is detecting the same issue multiple times!`);

                // Find and log all duplicates
                const keyCounts = new Map<string, number>();
                issueKeys.forEach(key => keyCounts.set(key, (keyCounts.get(key) || 0) + 1));

                keyCounts.forEach((count, key) => {
                    if (count > 1) {
                        console.log(`  🔴 Duplicate (${count}x): ${key}`);
                    }
                });
            }

            // Log all issues for debugging (only first 5)
            console.log(`📋 [EXEC] All ${issues.length} raw issues from CLI:`);
            issues.slice(0, 8).forEach((issue, idx) => {
                console.log(`  [${idx}] Line ${issue.line}, Col ${issue.column}: [${issue.rule_name}] ${issue.message} (${issue.severity})`);
            });
            if (issues.length > 8) {
                console.log(`  ... and ${issues.length - 8} more`);
            }
        }

        const diagnostics = issuesToDiagnostics(issues);

        // CRITICAL DEBUG: Track exact state of diagnostics
        console.log(`\n🔬 [DIAG-DEBUG] ===== DIAGNOSTIC LIFECYCLE TRACKING =====`);
        console.log(`🔬 [DIAG-DEBUG] File: ${document.fileName}`);
        console.log(`🔬 [DIAG-DEBUG] Raw issues from CLI: ${issues.length}`);
        console.log(`🔬 [DIAG-DEBUG] After dedup (issuesToDiagnostics): ${diagnostics.length}`);

        // Log all unique diagnostics being set
        if (diagnostics.length > 0) {
            console.log(`🔬 [DIAG-DEBUG] Diagnostics to be set:`);
            diagnostics.slice(0, 10).forEach((d, idx) => {
                console.log(`  [${idx}] Line ${d.range.start.line + 1}: "${d.message}" [${d.code}]`);
            });
            if (diagnostics.length > 10) {
                console.log(`  ... and ${diagnostics.length - 10} more`);
            }
        }

        // Check what's currently in the collection BEFORE we change it
        const beforeDelete = diagnosticCollection.get(document.uri);
        console.log(`🔬 [DIAG-DEBUG] Current diagnostics in collection (BEFORE delete): ${beforeDelete?.length || 0}`);
        if (beforeDelete && beforeDelete.length > 0) {
            console.log(`🔬 [DIAG-DEBUG] Existing diagnostics:`);
            beforeDelete.slice(0, 5).forEach((d, idx) => {
                console.log(`  [${idx}] Line ${d.range.start.line + 1}: "${d.message}" [${d.code}]`);
            });
        }

        // CRITICAL: Use delete() then set() to force VS Code to clear old diagnostics first
        console.log(`🔬 [DIAG-DEBUG] Calling diagnosticCollection.delete()...`);
        diagnosticCollection.delete(document.uri);

        const afterDelete = diagnosticCollection.get(document.uri);
        console.log(`🔬 [DIAG-DEBUG] After delete: ${afterDelete?.length || 0} diagnostics`);

        console.log(`🔬 [DIAG-DEBUG] Calling diagnosticCollection.set() with ${diagnostics.length} diagnostics...`);
        diagnosticCollection.set(document.uri, diagnostics);

        // IMMEDIATELY verify what was actually set
        const afterSet = diagnosticCollection.get(document.uri);
        console.log(`🔬 [DIAG-DEBUG] After set: ${afterSet?.length || 0} diagnostics in collection`);
        if (afterSet && afterSet.length > 0) {
            console.log(`🔬 [DIAG-DEBUG] Actual diagnostics now in collection:`);
            afterSet.forEach((d, idx) => {
                console.log(`  [${idx}] Line ${d.range.start.line + 1}: "${d.message}" [${d.code}]`);
            });
        }
        console.log(`🔬 [DIAG-DEBUG] ===== END TRACKING =====\n`);

    } catch (error) {
        console.error(`❌ [EXEC] Request #${requestId} failed:`, error);
    } finally {
        activeAnalysis.delete(filePath);
        console.log(`✅ [EXEC] Request #${requestId} completed. Active: ${activeAnalysis.size}`);

        // Check if there's a queued analysis waiting for this file
        const queued = analysisQueue.get(filePath);
        if (queued) {
            console.log(`🔄 [EXEC] Found queued request #${queued.requestId}, will execute after debounce`);
            // The timer is already set, it will auto-execute
        }
    }
}

function analyzeOpenRustFiles() {
    vscode.workspace.textDocuments.forEach(document => {
        if (document.languageId === 'rust') {
            analyzeDocument(document);
        }
    });
}

async function runGarbageHunterOnPath(filePath: string): Promise<GarbageIssue[]> {
    const config = vscode.workspace.getConfiguration('garbageHunter');

    // Smart language detection (with caching)
    const detectedLanguage = await detectFileLanguage(filePath);
    const configLanguage = config.get<string>('language');

    // Use smart detection if language is set to auto
    const language = configLanguage === 'auto' || !configLanguage ? detectedLanguage : configLanguage;

    console.log(`🔍 File: ${filePath}, Detected: ${detectedLanguage}, Using: ${language}`);

    // Build command arguments safely (prevent command injection)
    const cli = getCliCommand();
    const args: string[] = [
        escapeShellArg(filePath),
        '--format', 'json',
        '--lang', language
    ];

    // Add exclude patterns
    const excludePatterns = config.get<string[]>('excludePatterns', []);
    for (const pattern of excludePatterns) {
        args.push('--exclude', escapeShellArg(pattern));
    }

    // Add LLM options if enabled
    const llmEnabled = config.get<boolean>('llm.enabled', false);
    if (llmEnabled) {
        args.push('--llm');
        args.push('--llm-provider', config.get<string>('llm.provider', 'ollama'));

        const model = config.get<string>('llm.model', 'gemma4:e2b');
        if (model) {
            args.push('--llm-model', model);
        }

        const endpoint = config.get<string>('llm.endpoint', '');
        if (endpoint) {
            args.push('--llm-endpoint', endpoint);
        }

        const apiKey = config.get<string>('llm.apiKey', '');
        if (apiKey) {
            args.push('--llm-api-key', apiKey);
        }
    }

    const command = `${cli} ${args.join(' ')}`;
    console.log(`📝 Command: ${command}`);

    return new Promise((resolve, reject) => {
        exec(
            command,
            {
                cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath,
                timeout: 30000, // 30 second timeout to prevent hanging
            },
            (error, stdout, stderr) => {
            if (error) {
                console.error(`Command execution error: ${error.message}`);
                console.error(`Command: ${command}`);
                console.error(`Stderr: ${stderr}`);

                // If there's stdout, try to parse it (CLI may exit non-zero with valid output)
                if (stdout.trim() !== '') {
                    try {
                        const issues: GarbageIssue[] = JSON.parse(stdout);
                        resolve(issues);
                        return;
                    } catch (parseError) {
                        // Parse failed, continue to error handling
                    }
                }

                // Show actionable error to the user
                const errorMsg = error.message || '';
                if (errorMsg.includes('ENOENT') || errorMsg.includes('command not found') || errorMsg.includes('not found')) {
                    vscode.window.showErrorMessage(
                        '🗑️ Garbage Code Hunter: CLI not found. Run `cargo install garbage-code-hunter` and restart VS Code.',
                        'Copy Install Command'
                    ).then(selection => {
                        if (selection === 'Copy Install Command') {
                            vscode.env.clipboard.writeText('cargo install garbage-code-hunter');
                        }
                    });
                } else {
                    vscode.window.showErrorMessage(`🗑️ Garbage Code Hunter error: ${stderr || errorMsg}`);
                }

                resolve([]);
                return;
            }

            try {
                if (!stdout.trim()) {
                    resolve([]);
                    return;
                }

                const issues: GarbageIssue[] = JSON.parse(stdout);
                console.log(`✅ Successfully parsed ${issues.length} issues from CLI output`);
                resolve(issues);
            } catch (parseError) {
                reject(new Error(`Failed to parse output: ${parseError}`));
            }
        });
    });
}

/**
 * Run CLI with extra flags (e.g., --summary, --educational) and return raw stdout.
 */
async function runCliWithFlags(filePath: string, extraFlags: string[]): Promise<string> {
    const config = vscode.workspace.getConfiguration('garbageHunter');
    const language = config.get<string>('language', 'en-US');

    const cli = getCliCommand();
    const args: string[] = [
        escapeShellArg(filePath),
        '--lang', language
    ];

    // Add exclude patterns
    const excludePatterns = config.get<string[]>('excludePatterns', []);
    for (const pattern of excludePatterns) {
        args.push('--exclude', escapeShellArg(pattern));
    }

    // Add extra flags
    for (const flag of extraFlags) {
        args.push(flag);
    }

    const command = `${cli} ${args.join(' ')}`;

    return new Promise((resolve) => {
        exec(command, { cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath }, (error, stdout) => {
            // CLI may exit non-zero even with valid output
            resolve(stdout || '');
        });
    });
}

function shouldExcludeFile(filePath: string): boolean {
    const config = vscode.workspace.getConfiguration('garbageHunter');
    const excludePatterns = config.get<string[]>('excludePatterns', []);

    return excludePatterns.some(pattern => {
        const regex = new RegExp(pattern.replace(/\*\*/g, '.*').replace(/\*/g, '[^/]*'));
        return regex.test(filePath);
    });
}

function groupIssuesByFile(issues: GarbageIssue[]): Map<string, GarbageIssue[]> {
    const grouped = new Map<string, GarbageIssue[]>();

    for (const issue of issues) {
        const filePath = issue.file_path;
        if (!grouped.has(filePath)) {
            grouped.set(filePath, []);
        }
        grouped.get(filePath)!.push(issue);
    }

    return grouped;
}

function issuesToDiagnostics(issues: GarbageIssue[]): vscode.Diagnostic[] {
    // Deduplicate issues: same line + same message + same rule = duplicate
    const seen = new Set<string>();
    const uniqueIssues: GarbageIssue[] = [];
    let duplicateCount = 0;

    for (const issue of issues) {
        // Create unique key based on line, message, and rule
        const key = `${issue.line}|${issue.rule_name}|${issue.message}`;

        if (seen.has(key)) {
            duplicateCount++;
            console.log(`  ⚠️ [DEDUP] Duplicate detected: Line ${issue.line} [${issue.rule_name}] ${issue.message}`);
            continue;
        }

        seen.add(key);
        uniqueIssues.push(issue);
    }

    if (duplicateCount > 0) {
        console.log(`🔄 [DEDUP] Removed ${duplicateCount} duplicate(s) from ${issues.length} issue(s), keeping ${uniqueIssues.length}`);
    }

    return uniqueIssues.map(issue => {
        const line = Math.max(0, issue.line - 1); // VS Code uses 0-based line numbers
        const column = Math.max(0, issue.column - 1);

        // Create precise range highlighting the problematic token
        const range = new vscode.Range(
            new vscode.Position(line, column),
            new vscode.Position(line, column + getTokenLength(issue))
        );

        const severity = severityToVSCodeSeverity(issue.severity);

        const diagnostic = new vscode.Diagnostic(
            range,
            `🗑️ ${issue.message}`,
            severity
        );

        diagnostic.source = 'Garbage Hunter';
        diagnostic.code = issue.rule_name;

        return diagnostic;
    });
}

/**
 * Estimate token length based on rule type for precise highlighting.
 */
function getTokenLength(issue: GarbageIssue): number {
    switch (issue.rule_name) {
        case 'terrible-naming':
        case 'meaningless-naming':
        case 'single-letter-variable':
        case 'hungarian-notation':
        case 'abbreviation-abuse':
            return estimateVariableNameLength(issue.message);
        case 'unwrap-abuse':
            return 7; // "unwrap()" length
        case 'println-debugging':
            return 8; // "println!" length
        case 'magic-number':
            return estimateNumberLength(issue.message);
        default:
            return 5; // Default length
    }
}

function estimateVariableNameLength(message: string): number {
    const matches = message.match(/Variable '(\w+)'/);
    if (matches && matches[1]) {
        return matches[1].length;
    }
    return 4;
}

function estimateNumberLength(message: string): number {
    const matches = message.match(/(\d+(?:\.\d+)?)/);
    if (matches && matches[1]) {
        return matches[1].length;
    }
    return 2;
}

function severityToVSCodeSeverity(severity: string): vscode.DiagnosticSeverity {
    switch (severity) {
        case 'Nuclear':
            return vscode.DiagnosticSeverity.Error;
        case 'Spicy':
            return vscode.DiagnosticSeverity.Warning;
        case 'Mild':
        default:
            return vscode.DiagnosticSeverity.Information;
    }
}

/**
 * Show quality score summary for the workspace or current file.
 */
async function showQualityScore() {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    const editor = vscode.window.activeTextEditor;

    const targetPath = editor?.document.languageId === 'rust'
        ? editor.document.fileName
        : workspaceFolder?.uri.fsPath;

    if (!targetPath) {
        vscode.window.showWarningMessage('No Rust file or workspace folder found');
        return;
    }

    await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: "📊 Calculating quality score...",
        cancellable: false
    }, async () => {
        const output = await runCliWithFlags(targetPath, ['--summary']);

        outputChannel.clear();
        outputChannel.appendLine(output);
        outputChannel.show(true);

        // Extract score from output for notification
        const scoreMatch = output.match(/Score:\s*([\d.]+)\/100/);
        if (scoreMatch) {
            const score = parseFloat(scoreMatch[1]);
            const level = score <= 20 ? '🏆 Excellent' :
                         score <= 40 ? '👍 Good' :
                         score <= 60 ? '😐 Average' :
                         score <= 80 ? '😟 Poor' : '💀 Terrible';
            vscode.window.showInformationMessage(`Quality Score: ${score}/100 (${level})`);
        }
    });
}

/**
 * Show educational advice for detected issues.
 */
async function showEducationalAdvice() {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    const editor = vscode.window.activeTextEditor;

    const targetPath = editor?.document.languageId === 'rust'
        ? editor.document.fileName
        : workspaceFolder?.uri.fsPath;

    if (!targetPath) {
        vscode.window.showWarningMessage('No Rust file or workspace folder found');
        return;
    }

    await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: "🎓 Generating educational advice...",
        cancellable: false
    }, async () => {
        const output = await runCliWithFlags(targetPath, ['--educational', '--summary']);

        outputChannel.clear();
        outputChannel.appendLine(output);
        outputChannel.show(true);
    });
}

/**
 * Detect file language based on comment content (Chinese vs English).
 */
async function detectFileLanguage(filePath: string): Promise<string> {
    // Check cache first
    const cached = languageCache.get(filePath);
    if (cached) {
        return cached;
    }

    try {
        const fs = require('fs').promises;
        const content = await fs.readFile(filePath, 'utf8');

        // Detect Chinese characters only in comments
        const chineseRegex = /[一-鿿]/;
        const lines = content.split('\n');
        let hasChineseComments = false;

        for (const line of lines) {
            const trimmed = line.trim();
            // Check single-line comments
            if (trimmed.startsWith('//')) {
                if (chineseRegex.test(trimmed)) {
                    hasChineseComments = true;
                    break;
                }
            } else if (trimmed.startsWith('/*') || trimmed.includes('/*')) {
                // Check multi-line comment start
                if (chineseRegex.test(trimmed)) {
                    hasChineseComments = true;
                    break;
                }
            }
        }

        const result = hasChineseComments ? 'zh-CN' : 'en-US';
        languageCache.set(filePath, result);  // Cache the result
        return result;
    } catch (error) {
        return 'en-US'; // Default to English
    }
}

export function deactivate() {
    // Clear all pending analysis queues to prevent memory leaks
    for (const queued of analysisQueue.values()) {
        clearTimeout(queued.timer);
    }
    analysisQueue.clear();
    activeAnalysis.clear();
    languageCache.clear();

    if (diagnosticCollection) {
        diagnosticCollection.dispose();
    }


    if (outputChannel) {
        outputChannel.dispose();
    }
}
