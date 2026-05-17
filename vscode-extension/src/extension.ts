import * as vscode from 'vscode';
import { exec } from 'child_process';

const analysisQueue = new Map<string, {
    timer: NodeJS.Timeout;
    document: vscode.TextDocument;
    requestId: number;
}>();
const activeAnalysis = new Set<string>();
let globalRequestId = 0;
const languageCache = new Map<string, string>();
const DEBOUNCE_MS = 800;
const MAX_CACHE_SIZE = 100;

const SUPPORTED_LANGUAGES = new Set([
    'rust', 'python', 'javascript', 'typescript',
    'go', 'java', 'ruby', 'c', 'cpp',
]);

function escapeShellArg(arg: string): string {
    return `'${arg.replace(/'/g, "'\\''")}'`;
}

interface GarbageIssue {
    file_path: string;
    line: number;
    column: number;
    rule_name: string;
    message: string;
    severity: 'Mild' | 'Spicy' | 'Nuclear';
}

interface StyleIrJsonSummary {
    language: string;
    line_count: number;
    function_count: number;
    god_function_count: number;
    panic_call_count: number;
    naming_violation_count: number;
    deeply_nested_block_count: number;
    debug_call_count: number;
    excessive_param_count: number;
    unsafe_block_count: number;
    magic_number_count: number;
    over_engineering_count: number;
    code_smell_count: number;
    is_clean_signal_baseline: boolean;
    thresholds: {
        excessive_param_threshold: number;
        god_function_line_threshold: number;
    };
}

interface AnalyzeJsonReport {
    issues: GarbageIssue[];
    style_ir_summary?: StyleIrJsonSummary | null;
}

let diagnosticCollection: vscode.DiagnosticCollection;
let outputChannel: vscode.OutputChannel;
let decorationType: vscode.TextEditorDecorationType;

export function activate(context: vscode.ExtensionContext) {
    console.log('🗑️ Garbage Code Hunter is now active!');
    diagnosticCollection = vscode.languages.createDiagnosticCollection('garbage-hunter');
    outputChannel = vscode.window.createOutputChannel('Garbage Hunter');

    decorationType = vscode.window.createTextEditorDecorationType({
        backgroundColor: 'rgba(255, 100, 100, 0.08)',
        borderRadius: '3px',
        isWholeLine: false,
    });

    checkCliAvailability();
    registerCommands(context);
    registerFileWatchers(context);
    registerConfigurationWatcher(context);
    registerHoverProvider(context);

    setTimeout(() => { analyzeOpenFiles(); }, 1000);
    context.subscriptions.push(diagnosticCollection, outputChannel, decorationType);
}

function getCliCommand(): string {
    const config = vscode.workspace.getConfiguration('garbageHunter');
    return config.get<string>('cliPath', '') || 'garbage-code-hunter';
}

async function checkCliAvailability() {
    const cli = getCliCommand();
    exec(`${cli} --version`, (error, stdout) => {
        if (error) {
            const msg = error.message || '';
            if (msg.includes('ENOENT') || msg.includes('not found')) {
                vscode.window.showWarningMessage(
                    '🗑️ Garbage Code Hunter: CLI not found. Install with `cargo install garbage-code-hunter` or set `garbageHunter.cliPath`.',
                    'Open Settings'
                ).then(sel => {
                    if (sel === 'Open Settings')
                        vscode.commands.executeCommand('workbench.action.openSettings', 'garbageHunter.cliPath');
                });
            }
        } else {
            console.log(`🗑️ CLI found: ${stdout.trim()}`);
        }
    });
}

function registerCommands(context: vscode.ExtensionContext) {
    context.subscriptions.push(
        vscode.commands.registerCommand('garbageHunter.analyzeFile', () => analyzeCurrentFile()),
        vscode.commands.registerCommand('garbageHunter.analyzeWorkspace', () => analyzeWorkspace()),
        vscode.commands.registerCommand('garbageHunter.clearDiagnostics', () => {
            diagnosticCollection.clear();
            vscode.window.showInformationMessage('🧹 All roasts cleared!');
        }),
        vscode.commands.registerCommand('garbageHunter.showScore', () => showQualityScore()),
        vscode.commands.registerCommand('garbageHunter.showEducational', () => showEducationalAdvice()),
    );
}

function registerFileWatchers(context: vscode.ExtensionContext) {
    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument((doc) => {
            const config = vscode.workspace.getConfiguration('garbageHunter');
            if (config.get('enableRealTimeAnalysis', true) && SUPPORTED_LANGUAGES.has(doc.languageId)) {
                analyzeDocument(doc);
            }
        })
    );
}

function registerConfigurationWatcher(context: vscode.ExtensionContext) {
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration((event) => {
            if (event.affectsConfiguration('garbageHunter')) {
                languageCache.clear();
                analyzeOpenFiles();
            }
        })
    );
}

function registerHoverProvider(context: vscode.ExtensionContext) {
    context.subscriptions.push(
        vscode.languages.registerHoverProvider(Array.from(SUPPORTED_LANGUAGES).map(id => ({ language: id })), {
            provideHover(document, position) {
                const diags = diagnosticCollection.get(document.uri);
                if (!diags) return null;
                for (const d of diags) {
                    if (d.range.contains(position)) {
                        return new vscode.Hover({
                            language: 'text',
                            value: `🗑️ **${d.code}**\n\n${d.message.replace('🗑️ ', '')}\n\nSeverity: ${severityLabel(d.severity)}`
                        });
                    }
                }
                return null;
            }
        })
    );
}

function severityLabel(sev: vscode.DiagnosticSeverity): string {
    switch (sev) {
        case vscode.DiagnosticSeverity.Error: return '🔥 Nuclear';
        case vscode.DiagnosticSeverity.Warning: return '🌶️ Spicy';
        default: return '😐 Mild';
    }
}

async function analyzeCurrentFile() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) { vscode.window.showWarningMessage('No active file'); return; }
    await analyzeDocument(editor.document);
}

async function analyzeWorkspace() {
    const wf = vscode.workspace.workspaceFolders?.[0];
    if (!wf) { vscode.window.showWarningMessage('No workspace'); return; }

    await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: "🔥 Roasting your codebase...",
        cancellable: false
    }, async (progress) => {
        progress.report({ increment: 0 });
        const issues = await runGarbageHunterOnPath(wf.uri.fsPath);
        diagnosticCollection.clear();
        for (const [fp, fi] of groupIssuesByFile(issues)) {
            diagnosticCollection.set(vscode.Uri.file(fp), issuesToDiagnostics(fi));
        }
        progress.report({ increment: 100 });
        vscode.window.showWarningMessage(`🗑️ Found ${issues.length} issues across ${new Set(issues.map(i => i.file_path)).size} files`);
    });
}

async function analyzeDocument(document: vscode.TextDocument) {
    if (shouldExcludeFile(document.uri.fsPath)) return;
    const filePath = document.uri.fsPath;
    const currentRequestId = ++globalRequestId;
    const existing = analysisQueue.get(filePath);
    if (existing) { clearTimeout(existing.timer); analysisQueue.delete(filePath); }
    if (activeAnalysis.has(filePath)) {
        const timer = setTimeout(() => {
            analysisQueue.delete(filePath);
            analyzeDocument(document);
        }, DEBOUNCE_MS * 2);
        analysisQueue.set(filePath, { timer, document, requestId: currentRequestId });
        return;
    }
    const timer = setTimeout(async () => {
        analysisQueue.delete(filePath);
        await executeAnalysis(document, currentRequestId);
    }, DEBOUNCE_MS);
    analysisQueue.set(filePath, { timer, document, requestId: currentRequestId });
}

async function executeAnalysis(document: vscode.TextDocument, requestId: number) {
    if (activeAnalysis.has(document.uri.fsPath)) return;
    activeAnalysis.add(document.uri.fsPath);
    try {
        const issues = await runGarbageHunterOnPath(document.fileName);
        const diags = issuesToDiagnostics(issues);
        diagnosticCollection.delete(document.uri);
        diagnosticCollection.set(document.uri, diags);
        updateInlineDecorations(document, issues);
    } catch (e) {
        console.error(e);
    } finally {
        activeAnalysis.delete(document.uri.fsPath);
        const q = analysisQueue.get(document.uri.fsPath);
        if (q) analyzeDocument(q.document);
    }
}

function updateInlineDecorations(document: vscode.TextDocument, issues: GarbageIssue[]) {
    const editor = vscode.window.visibleTextEditors.find(e => e.document.uri.toString() === document.uri.toString());
    if (!editor) return;
    const decorations = issues.map(i => {
        const line = Math.max(0, i.line - 1);
        const col = Math.max(0, i.column - 1);
        const range = new vscode.Range(line, col, line, col + 5);
        return { range, hoverMessage: `🗑️ ${i.message}` };
    });
    editor.setDecorations(decorationType, decorations);
}

function analyzeOpenFiles() {
    vscode.workspace.textDocuments.forEach(doc => {
        if (SUPPORTED_LANGUAGES.has(doc.languageId)) analyzeDocument(doc);
    });
}

async function runGarbageHunterOnPath(filePath: string): Promise<GarbageIssue[]> {
    const config = vscode.workspace.getConfiguration('garbageHunter');
    const language = config.get<string>('language', 'en-US') === 'auto'
        ? await detectFileLanguage(filePath) : config.get<string>('language', 'en-US');
    const cli = getCliCommand();
    const args = [escapeShellArg(filePath), '--format', 'json', '--lang', language];
    for (const p of config.get<string[]>('excludePatterns', [])) args.push('--exclude', escapeShellArg(p));
    if (config.get<boolean>('llm.enabled', false)) {
        args.push('--llm');
        args.push('--llm-provider', config.get<string>('llm.provider', 'ollama'));
        const m = config.get<string>('llm.model', ''); if (m) args.push('--llm-model', m);
        const e = config.get<string>('llm.endpoint', ''); if (e) args.push('--llm-endpoint', e);
        const k = config.get<string>('llm.apiKey', ''); if (k) args.push('--llm-api-key', k);
    }
    return new Promise((resolve) => {
        exec(`${cli} ${args.join(' ')}`, { cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath, timeout: 30000 },
            (_error, stdout) => {
                if (!stdout.trim()) { resolve([]); return; }
                try { resolve(normalizeAnalyzeJson(JSON.parse(stdout))); }
                catch { resolve([]); }
            });
    });
}

async function runCliWithFlags(filePath: string, extraFlags: string[]): Promise<string> {
    const config = vscode.workspace.getConfiguration('garbageHunter');
    const language = config.get<string>('language', 'en-US');
    const cli = getCliCommand();
    const args = [escapeShellArg(filePath), '--lang', language, ...extraFlags];
    return new Promise((resolve) => {
        exec(`${cli} ${args.join(' ')}`, { timeout: 30000 }, (_e, stdout) => resolve(stdout || ''));
    });
}

function shouldExcludeFile(filePath: string): boolean {
    const patterns = vscode.workspace.getConfiguration('garbageHunter').get<string[]>('excludePatterns', []);
    return patterns.some(p => new RegExp(p.replace(/\*\*/g, '.*').replace(/\*/g, '[^/]*')).test(filePath));
}

function groupIssuesByFile(issues: GarbageIssue[]): Map<string, GarbageIssue[]> {
    const m = new Map<string, GarbageIssue[]>();
    for (const i of issues) {
        if (!m.has(i.file_path)) m.set(i.file_path, []);
        m.get(i.file_path)!.push(i);
    }
    return m;
}

function normalizeAnalyzeJson(output: unknown): GarbageIssue[] {
    if (Array.isArray(output)) {
        return output as GarbageIssue[];
    }
    if (output && typeof output === 'object') {
        const report = output as AnalyzeJsonReport;
        if (Array.isArray(report.issues)) {
            return report.issues;
        }
    }
    return [];
}

function issuesToDiagnostics(issues: GarbageIssue[]): vscode.Diagnostic[] {
    const seen = new Set<string>();
    return issues.filter(i => {
        const k = `${i.line}|${i.rule_name}|${i.message}`;
        if (seen.has(k)) return false;
        seen.add(k); return true;
    }).map(i => {
        const line = Math.max(0, i.line - 1);
        const col = Math.max(0, i.column - 1);
        const range = new vscode.Range(line, col, line, col + Math.max(5, i.message.length > 60 ? 5 : 3));
        const sev = i.severity === 'Nuclear' ? vscode.DiagnosticSeverity.Error
            : i.severity === 'Spicy' ? vscode.DiagnosticSeverity.Warning
            : vscode.DiagnosticSeverity.Information;
        const d = new vscode.Diagnostic(range, `🗑️ ${i.message}`, sev);
        d.source = 'Garbage Hunter';
        d.code = i.rule_name;
        return d;
    });
}

async function showQualityScore() {
    const target = vscode.window.activeTextEditor?.document.fileName
        || vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (!target) return;
    const out = await runCliWithFlags(target, ['--summary']);
    outputChannel.clear();
    outputChannel.appendLine(out);
    outputChannel.show(true);
}

async function showEducationalAdvice() {
    const target = vscode.window.activeTextEditor?.document.fileName
        || vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (!target) return;
    const out = await runCliWithFlags(target, ['--educational', '--summary']);
    outputChannel.clear();
    outputChannel.appendLine(out);
    outputChannel.show(true);
}

async function detectFileLanguage(filePath: string): Promise<string> {
    const cached = languageCache.get(filePath);
    if (cached) return cached;
    try {
        const fs = require('fs').promises;
        const content = await fs.readFile(filePath, 'utf8');
        const cn = /[一-鿿]/;
        for (const line of content.split('\n')) {
            const t = line.trim();
            if ((t.startsWith('//') || t.startsWith('/*') || t.includes('/*')) && cn.test(t)) {
                if (languageCache.size >= MAX_CACHE_SIZE) languageCache.delete(languageCache.keys().next().value);
                languageCache.set(filePath, 'zh-CN');
                return 'zh-CN';
            }
        }
        languageCache.set(filePath, 'en-US');
        return 'en-US';
    } catch { return 'en-US'; }
}

export function deactivate() {
    for (const q of analysisQueue.values()) clearTimeout(q.timer);
    analysisQueue.clear();
    activeAnalysis.clear();
    languageCache.clear();
    diagnosticCollection?.dispose();
    outputChannel?.dispose();
}
