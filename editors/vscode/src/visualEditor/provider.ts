import * as vscode from 'vscode';
import { getNonce, getWebviewContent } from './webview';

/**
 * Custom editor provider for Naze visual editor.
 * Provides a block-based visual editor alongside the source view.
 */
export class NazeVisualEditorProvider implements vscode.CustomTextEditorProvider {
  public static readonly viewType = 'naze.visualEditor';

  public static register(context: vscode.ExtensionContext): vscode.Disposable {
    const provider = new NazeVisualEditorProvider(context);
    return vscode.window.registerCustomEditorProvider(
      NazeVisualEditorProvider.viewType,
      provider,
      {
        webviewOptions: {
          retainContextWhenHidden: true,
        },
        supportsMultipleEditorsPerDocument: false,
      }
    );
  }

  constructor(private readonly context: vscode.ExtensionContext) {}

  public async resolveCustomTextEditor(
    document: vscode.TextDocument,
    webviewPanel: vscode.WebviewPanel,
    _token: vscode.CancellationToken
  ): Promise<void> {
    // Set up the webview
    webviewPanel.webview.options = {
      enableScripts: true,
      localResourceRoots: [
        vscode.Uri.joinPath(this.context.extensionUri, 'webview', 'dist'),
        vscode.Uri.joinPath(this.context.extensionUri, 'media'),
      ],
    };

    // Get the webview content
    webviewPanel.webview.html = getWebviewContent(
      webviewPanel.webview,
      this.context.extensionUri
    );

    // Send initial document content to webview
    const updateWebview = () => {
      webviewPanel.webview.postMessage({
        type: 'update',
        content: document.getText(),
        fileName: document.fileName,
      });
    };

    // Handle messages from the webview
    webviewPanel.webview.onDidReceiveMessage(
      (message) => {
        switch (message.type) {
          case 'edit':
            // Apply edit from visual editor to source
            this.applyEdit(document, message.edit);
            break;
          case 'ready':
            // Webview is ready, send initial content
            updateWebview();
            break;
          case 'ai-request':
            // Handle AI request (TODO: implement)
            this.handleAIRequest(webviewPanel.webview, message);
            break;
        }
      },
      undefined,
      []
    );

    // Update webview when document changes
    const changeDocumentSubscription = vscode.workspace.onDidChangeTextDocument(
      (e) => {
        if (e.document.uri.toString() === document.uri.toString()) {
          updateWebview();
        }
      }
    );

    // Clean up when editor is closed
    webviewPanel.onDidDispose(() => {
      changeDocumentSubscription.dispose();
    });

    // Send initial content
    updateWebview();
  }

  /**
   * Apply an edit from the visual editor to the source document.
   */
  private async applyEdit(
    document: vscode.TextDocument,
    edit: { start: number; end: number; text: string }
  ) {
    const workspaceEdit = new vscode.WorkspaceEdit();
    const startPos = document.positionAt(edit.start);
    const endPos = document.positionAt(edit.end);
    workspaceEdit.replace(
      document.uri,
      new vscode.Range(startPos, endPos),
      edit.text
    );
    await vscode.workspace.applyEdit(workspaceEdit);
  }

  /**
   * Handle AI request from the visual editor.
   */
  private async handleAIRequest(
    webview: vscode.Webview,
    message: { prompt: string; context?: string }
  ) {
    const config = vscode.workspace.getConfiguration('naze');
    const aiEnabled = config.get<boolean>('ai.enabled', true);

    if (!aiEnabled) {
      webview.postMessage({
        type: 'ai-response',
        error: 'AI features are disabled. Enable them in settings.',
      });
      return;
    }

    const apiKey =
      config.get<string>('ai.apiKey') || process.env.ANTHROPIC_API_KEY;

    if (!apiKey) {
      webview.postMessage({
        type: 'ai-response',
        error:
          'No API key configured. Set naze.ai.apiKey in settings or ANTHROPIC_API_KEY environment variable.',
      });
      return;
    }

    // TODO: Implement actual Claude API call
    // For now, return a placeholder response
    webview.postMessage({
      type: 'ai-response',
      content: `-- AI-generated code will appear here\n-- Prompt: ${message.prompt}`,
    });
  }
}

/**
 * Get a nonce for inline scripts (CSP).
 */
export { getNonce };
