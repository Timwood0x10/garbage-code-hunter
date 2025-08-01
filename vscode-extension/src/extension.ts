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
    const language = config.get<string>('language', 'en-US');
    const excludePatterns = config.get<string[]>('excludePatterns', []);
    
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
        
        const range = new vscode.Range(
            new vscode.Position(line, column),
            new vscode.Position(line, column + 10) // Approximate word length
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

        const decoration: vscode.DecorationOptions = {
            range: diagnostic.range,
            renderOptions: {
                after: {
                    contentText: ` ${message}`,
                    color: getSeverityColor(diagnostic.severity),
                    fontStyle: 'italic',
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

export function deactivate() {
    if (diagnosticCollection) {
        diagnosticCollection.dispose();
    }
    if (inlineDecorationType) {
        inlineDecorationType.dispose();
    }
}