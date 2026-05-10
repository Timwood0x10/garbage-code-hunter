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
const MAX_CACHE_SIZE = 100;  // Maximum number of files to cache

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
            // Clear language cache when config changes
            languageCache.clear();
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

    const currentRequestId = ++globalRequestId;

    // Cancel any existing queued analysis for this file
    const existing = analysisQueue.get(filePath);
    if (existing) {
        clearTimeout(existing.timer);
        analysisQueue.delete(filePath);
    }

    // If already actively analyzing this file, queue a new analysis after completion
    if (activeAnalysis.has(filePath)) {
        const timer = setTimeout(() => {
            analysisQueue.delete(filePath);
            analyzeDocument(document);
        }, DEBOUNCE_MS * 2);

        analysisQueue.set(filePath, { timer, document, requestId: currentRequestId });
        return;
    }

    // Debounce: wait before actually analyzing
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
        return;
    }

    activeAnalysis.add(filePath);

    try {
        const issues = await runGarbageHunterOnPath(document.fileName);
        const diagnostics = issuesToDiagnostics(issues);

        diagnosticCollection.delete(document.uri);
        diagnosticCollection.set(document.uri, diagnostics);

    } catch (error) {
        console.error(`Garbage Hunter analysis failed:`, error);
    } finally {
        activeAnalysis.delete(filePath);

        const queued = analysisQueue.get(filePath);
        if (queued) {
            analyzeDocument(queued.document);
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

    const detectedLanguage = await detectFileLanguage(filePath);
    const configLanguage = config.get<string>('language');

    // Use smart detection if language is set to auto
    const language = configLanguage === 'auto' || !configLanguage ? detectedLanguage : configLanguage;

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

    return new Promise((resolve, reject) => {
        exec(
            command,
            {
                cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath,
                timeout: 30000,
            },
            (error, stdout, stderr) => {
            if (error) {
                console.error(`Garbage Hunter command error: ${error.message}`);

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
    const detectedLanguage = await detectFileLanguage(filePath);
    const configLanguage = config.get<string>('language');

    // Use smart detection if language is set to auto (consistent with runGarbageHunterOnPath)
    const language = configLanguage === 'auto' || !configLanguage ? detectedLanguage : configLanguage;

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
        exec(command, {
            cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath,
            timeout: 30000
        }, (error, stdout) => {
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
    const seen = new Set<string>();
    const uniqueIssues: GarbageIssue[] = [];

    for (const issue of issues) {
        const key = `${issue.line}|${issue.rule_name}|${issue.message}`;

        if (seen.has(key)) {
            continue;
        }

        seen.add(key);
        uniqueIssues.push(issue);
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

        // Enforce cache size limit
        if (languageCache.size >= MAX_CACHE_SIZE) {
            const firstKey = languageCache.keys().next().value;
            if (firstKey) {
                languageCache.delete(firstKey);
            }
        }

        languageCache.set(filePath, result);
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
