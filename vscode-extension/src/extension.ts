import * as vscode from 'vscode';
import { exec } from 'child_process';
import * as path from 'path';

// 定义问题接口，对应 Rust 代码的 CodeIssue
interface GarbageIssue {
    file_path: string;
    line: number;
    column: number;
    rule_name: string;
    message: string;
    severity: 'Mild' | 'Spicy' | 'Nuclear';
}

// 全局诊断集合
let diagnosticCollection: vscode.DiagnosticCollection;

// 内联装饰器类型
let inlineDecorationType: vscode.TextEditorDecorationType;

export function activate(context: vscode.ExtensionContext) {
    console.log('🗑️ Garbage Code Hunter is now active!');

    // 创建诊断集合
    diagnosticCollection = vscode.languages.createDiagnosticCollection('garbage-hunter');
    
    // 创建内联装饰器类型（类似 ErrorLens）
    createInlineDecorationType();

    // 注册命令
    registerCommands(context);

    // 监听文件变化
    registerFileWatchers(context);

    // 监听配置变化
    registerConfigurationWatcher(context);

    // 分析当前打开的 Rust 文件
    analyzeOpenRustFiles();

    context.subscriptions.push(diagnosticCollection);
}

function createInlineDecorationType() {
    inlineDecorationType = vscode.window.createTextEditorDecorationType({
        after: {
            margin: '0 0 0 1em',
            fontStyle: 'italic',
        }
    });
}

function registerCommands(context: vscode.ExtensionContext) {
    // 分析当前文件
    const analyzeFileCommand = vscode.commands.registerCommand(
        'garbageHunter.analyzeFile',
        () => analyzeCurrentFile()
    );

    // 分析整个工作区
    const analyzeWorkspaceCommand = vscode.commands.registerCommand(
        'garbageHunter.analyzeWorkspace',
        () => analyzeWorkspace()
    );

    // 清除所有诊断
    const clearDiagnosticsCommand = vscode.commands.registerCommand(
        'garbageHunter.clearDiagnostics',
        () => {
            diagnosticCollection.clear();
            clearInlineDecorations();
            vscode.window.showInformationMessage('🧹 All roasts cleared!');
        }
    );

    context.subscriptions.push(
        analyzeFileCommand,
        analyzeWorkspaceCommand,
        clearDiagnosticsCommand
    );
}

function registerFileWatchers(context: vscode.ExtensionContext) {
    // 监听文件保存
    const onSaveListener = vscode.workspace.onDidSaveTextDocument((document) => {
        const config = vscode.workspace.getConfiguration('garbageHunter');
        if (config.get('enableRealTimeAnalysis', true) && document.languageId === 'rust') {
            analyzeDocument(document);
        }
    });

    // 监听活动编辑器变化
    const onActiveEditorChangeListener = vscode.window.onDidChangeActiveTextEditor((editor) => {
        if (editor && editor.document.languageId === 'rust') {
            // 更新内联装饰
            updateInlineDecorations(editor);
        }
    });

    context.subscriptions.push(onSaveListener, onActiveEditorChangeListener);
}

function registerConfigurationWatcher(context: vscode.ExtensionContext) {
    const configWatcher = vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration('garbageHunter')) {
            // 重新创建装饰器类型
            createInlineDecorationType();
            
            // 重新分析所有打开的文件
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
            
            // 清除之前的诊断
            diagnosticCollection.clear();
            
            // 按文件分组处理问题
            const issuesByFile = groupIssuesByFile(issues);
            
            for (const [filePath, fileIssues] of issuesByFile) {
                const uri = vscode.Uri.file(filePath);
                const diagnostics = issuesToDiagnostics(fileIssues);
                diagnosticCollection.set(uri, diagnostics);
            }

            // 更新内联装饰
            updateAllInlineDecorations();

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
            vscode.window.showErrorMessage(`Analysis failed: ${error}`);
        }
    });
}

async function analyzeDocument(document: vscode.TextDocument) {
    if (shouldExcludeFile(document.uri.fsPath)) {
        return;
    }

    try {
        const issues = await runGarbageHunterOnPath(document.fileName);
        const diagnostics = issuesToDiagnostics(issues);
        
        diagnosticCollection.set(document.uri, diagnostics);
        
        // 更新内联装饰
        const editor = vscode.window.activeTextEditor;
        if (editor && editor.document.uri.toString() === document.uri.toString()) {
            updateInlineDecorations(editor);
        }
        
    } catch (error) {
        console.error('Analysis error:', error);
        // 静默失败，不打扰用户
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
    const excludePatterns = config.get<string[]>('excludePatterns', []);
    
    // 智能检测文件语言
    const detectedLanguage = await detectFileLanguage(filePath);
    const configLanguage = config.get<string>('language');
    
    // 如果用户没有手动设置语言，使用智能检测
    const language = configLanguage === 'auto' || !configLanguage ? detectedLanguage : configLanguage;
    
    console.log(`🔍 File: ${filePath}, Detected: ${detectedLanguage}, Using: ${language}`);
    
    // 构建命令
    let command = `garbage-code-hunter "${filePath}" --format json --lang ${language}`;
    
    // 添加排除模式
    if (excludePatterns.length > 0) {
        const excludeArgs = excludePatterns.map(pattern => `--exclude "${pattern}"`).join(' ');
        command += ` ${excludeArgs}`;
    }

    return new Promise((resolve, reject) => {
        exec(command, { cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath }, (error, stdout, stderr) => {
            if (error) {
                // 如果是因为没有找到问题而退出，返回空数组
                if (error.code === 0 || stdout.trim() === '') {
                    resolve([]);
                    return;
                }
                reject(new Error(`Command failed: ${error.message}`));
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
    return issues.map(issue => {
        const line = Math.max(0, issue.line - 1); // VS Code uses 0-based line numbers
        const column = Math.max(0, issue.column - 1);
        
        // 创建更精确的范围，只高亮问题变量/代码
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

// 根据规则类型估算 token 长度
function getTokenLength(issue: GarbageIssue): number {
    // 根据不同的规则类型返回合适的长度
    switch (issue.rule_name) {
        case 'terrible-naming':
        case 'meaningless-naming':
        case 'single-letter-variable':
        case 'hungarian-notation':
        case 'abbreviation-abuse':
            // 变量名相关问题，估算变量名长度
            return estimateVariableNameLength(issue.message);
        case 'unwrap-abuse':
            return 7; // "unwrap()" 的长度
        case 'println-debugging':
            return 8; // "println!" 的长度
        case 'magic-number':
            return estimateNumberLength(issue.message);
        default:
            return 5; // 默认长度
    }
}

function estimateVariableNameLength(message: string): number {
    // 从消息中提取变量名
    const matches = message.match(/Variable '(\w+)'/);
    if (matches && matches[1]) {
        return matches[1].length;
    }
    
    // 如果无法提取，返回常见变量名长度
    return 4;
}

function estimateNumberLength(message: string): number {
    // 从消息中提取数字
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

function updateInlineDecorations(editor: vscode.TextEditor) {
    const config = vscode.workspace.getConfiguration('garbageHunter');
    if (!config.get('showInlineMessages', true)) {
        return;
    }

    const diagnostics = diagnosticCollection.get(editor.document.uri);
    if (!diagnostics || diagnostics.length === 0) {
        editor.setDecorations(inlineDecorationType, []);
        return;
    }

    const maxLength = config.get<number>('maxInlineMessageLength', 100);
    const decorations: vscode.DecorationOptions[] = [];

    for (const diagnostic of diagnostics) {
        let message = diagnostic.message;
        if (message.length > maxLength) {
            message = message.substring(0, maxLength - 3) + '...';
        }

        // ErrorLens 风格：在行尾显示消息，而不是在问题位置
        const line = diagnostic.range.start.line;
        const lineText = editor.document.lineAt(line).text;
        const endOfLinePosition = new vscode.Position(line, lineText.length);
        
        const decoration: vscode.DecorationOptions = {
            range: new vscode.Range(endOfLinePosition, endOfLinePosition),
            renderOptions: {
                after: {
                    contentText: ` ${message}`,
                    color: getSeverityColor(diagnostic.severity),
                    fontStyle: 'italic',
                    margin: '0 0 0 1em',
                }
            }
        };

        decorations.push(decoration);
    }

    editor.setDecorations(inlineDecorationType, decorations);
}

function getSeverityColor(severity: vscode.DiagnosticSeverity): string {
    switch (severity) {
        case vscode.DiagnosticSeverity.Error:
            return '#ff4444';
        case vscode.DiagnosticSeverity.Warning:
            return '#ff8800';
        case vscode.DiagnosticSeverity.Information:
        default:
            return '#4488ff';
    }
}

function updateAllInlineDecorations() {
    vscode.window.visibleTextEditors.forEach(editor => {
        if (editor.document.languageId === 'rust') {
            updateInlineDecorations(editor);
        }
    });
}

function clearInlineDecorations() {
    vscode.window.visibleTextEditors.forEach(editor => {
        editor.setDecorations(inlineDecorationType, []);
    });
}

// 智能检测文件中的语言（基于注释内容）
async function detectFileLanguage(filePath: string): Promise<string> {
    try {
        const fs = require('fs').promises;
        const content = await fs.readFile(filePath, 'utf8');
        
        // 检测中文字符 - 只在注释中检测
        const chineseRegex = /[\u4e00-\u9fff]/;
        const lines = content.split('\n');
        let hasChineseComments = false;
        
        for (const line of lines) {
            const trimmed = line.trim();
            // 只检查注释行
            if (trimmed.startsWith('//')) {
                // 单行注释
                if (chineseRegex.test(trimmed)) {
                    hasChineseComments = true;
                    break;
                }
            } else if (trimmed.startsWith('/*') || trimmed.includes('/*')) {
                // 多行注释开始
                if (chineseRegex.test(trimmed)) {
                    hasChineseComments = true;
                    break;
                }
            }
        }
        
        return hasChineseComments ? 'zh-CN' : 'en-US';
    } catch (error) {
        return 'en-US'; // 默认英文
    }
}

export function deactivate() {
    if (diagnosticCollection) {
        diagnosticCollection.dispose();
    }
    if (inlineDecorationType) {
        inlineDecorationType.dispose();
    }
}