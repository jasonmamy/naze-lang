import * as vscode from 'vscode';

/**
 * Generate a nonce for Content Security Policy.
 */
export function getNonce(): string {
  let text = '';
  const possible =
    'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  for (let i = 0; i < 32; i++) {
    text += possible.charAt(Math.floor(Math.random() * possible.length));
  }
  return text;
}

/**
 * Get the HTML content for the visual editor webview.
 */
export function getWebviewContent(
  webview: vscode.Webview,
  extensionUri: vscode.Uri
): string {
  const nonce = getNonce();

  // URI for the webview's bundled JavaScript
  const scriptUri = webview.asWebviewUri(
    vscode.Uri.joinPath(extensionUri, 'webview', 'dist', 'index.js')
  );

  // URI for the webview's bundled CSS
  const styleUri = webview.asWebviewUri(
    vscode.Uri.joinPath(extensionUri, 'webview', 'dist', 'index.css')
  );

  // Fallback: If the webview bundle doesn't exist yet, show a placeholder
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="Content-Security-Policy" content="
    default-src 'none';
    style-src ${webview.cspSource} 'unsafe-inline';
    script-src 'nonce-${nonce}';
    img-src ${webview.cspSource} data:;
    font-src ${webview.cspSource};
  ">
  <title>Naze Visual Editor</title>
  <link href="${styleUri}" rel="stylesheet">
  <style>
    body {
      font-family: var(--vscode-font-family);
      color: var(--vscode-foreground);
      background: var(--vscode-editor-background);
      margin: 0;
      padding: 0;
      height: 100vh;
      display: flex;
      flex-direction: column;
    }

    .loading {
      display: flex;
      align-items: center;
      justify-content: center;
      height: 100%;
      flex-direction: column;
      gap: 16px;
    }

    .loading-spinner {
      width: 32px;
      height: 32px;
      border: 3px solid var(--vscode-progressBar-background);
      border-top-color: var(--vscode-button-background);
      border-radius: 50%;
      animation: spin 1s linear infinite;
    }

    @keyframes spin {
      to { transform: rotate(360deg); }
    }

    #app {
      flex: 1;
      display: flex;
      flex-direction: column;
    }

    .toolbar {
      padding: 8px 16px;
      border-bottom: 1px solid var(--vscode-panel-border);
      display: flex;
      align-items: center;
      gap: 8px;
      background: var(--vscode-sideBar-background);
    }

    .toolbar-title {
      font-weight: 600;
      flex: 1;
    }

    .editor-container {
      flex: 1;
      display: flex;
      overflow: hidden;
    }

    .block-editor {
      flex: 1;
      overflow: auto;
      padding: 16px;
    }

    .preview-panel {
      width: 400px;
      border-left: 1px solid var(--vscode-panel-border);
      background: var(--vscode-editor-background);
    }

    .ai-panel {
      border-top: 1px solid var(--vscode-panel-border);
      padding: 12px;
      background: var(--vscode-sideBar-background);
    }

    .ai-input {
      display: flex;
      gap: 8px;
    }

    .ai-input input {
      flex: 1;
      padding: 8px 12px;
      background: var(--vscode-input-background);
      color: var(--vscode-input-foreground);
      border: 1px solid var(--vscode-input-border);
      border-radius: 4px;
    }

    .ai-input button {
      padding: 8px 16px;
      background: var(--vscode-button-background);
      color: var(--vscode-button-foreground);
      border: none;
      border-radius: 4px;
      cursor: pointer;
    }

    .ai-input button:hover {
      background: var(--vscode-button-hoverBackground);
    }

    .voice-btn {
      padding: 8px;
      background: var(--vscode-button-secondaryBackground);
      color: var(--vscode-button-secondaryForeground);
      border: none;
      border-radius: 4px;
      cursor: not-allowed;
      opacity: 0.7;
    }

    .voice-btn:hover::after {
      content: 'Coming Soon';
      position: absolute;
      background: var(--vscode-editorWidget-background);
      padding: 4px 8px;
      border-radius: 4px;
      font-size: 12px;
      margin-top: 24px;
      margin-left: -40px;
    }
  </style>
</head>
<body>
  <div id="app">
    <div class="loading">
      <div class="loading-spinner"></div>
      <p>Loading Naze Visual Editor...</p>
      <p style="font-size: 12px; opacity: 0.7;">Build the webview first: cd editors/vscode/webview && npm run build</p>
    </div>
  </div>

  <script src="${scriptUri}" nonce="${nonce}"></script>
</body>
</html>`;
}
