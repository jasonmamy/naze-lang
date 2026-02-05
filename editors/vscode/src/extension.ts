import * as vscode from 'vscode';
import * as path from 'path';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

import { NazeVisualEditorProvider } from './visualEditor/provider';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
  console.log('Naze extension activating...');

  // Start the language server (async - don't block activation)
  startLanguageServer(context);

  // Register the visual editor provider
  context.subscriptions.push(
    NazeVisualEditorProvider.register(context)
  );

  // Register commands
  context.subscriptions.push(
    vscode.commands.registerCommand('naze.openVisualEditor', () => {
      const activeEditor = vscode.window.activeTextEditor;
      if (activeEditor && activeEditor.document.languageId === 'naze') {
        vscode.commands.executeCommand(
          'vscode.openWith',
          activeEditor.document.uri,
          'naze.visualEditor'
        );
      } else {
        vscode.window.showWarningMessage('Please open a .naze file first');
      }
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('naze.restartServer', async () => {
      if (client) {
        await client.stop();
      }
      await startLanguageServer(context);
      vscode.window.showInformationMessage('Naze language server restarted');
    })
  );

  // Code action commands
  context.subscriptions.push(
    vscode.commands.registerCommand('naze.wrapInColumn', async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) return;

      const selection = editor.selection;
      const text = editor.document.getText(selection);

      if (text) {
        const indentMatch = text.match(/^(\s*)/);
        const indent = indentMatch ? indentMatch[1] : '';
        const innerIndent = indent + '  ';

        const wrapped = `${indent}column gap: 16px {\n${text.split('\n').map(line => innerIndent + line.trimStart()).join('\n')}\n${indent}}`;

        await editor.edit(editBuilder => {
          editBuilder.replace(selection, wrapped);
        });
      }
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('naze.wrapInRow', async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) return;

      const selection = editor.selection;
      const text = editor.document.getText(selection);

      if (text) {
        const indentMatch = text.match(/^(\s*)/);
        const indent = indentMatch ? indentMatch[1] : '';
        const innerIndent = indent + '  ';

        const wrapped = `${indent}row gap: 8px {\n${text.split('\n').map(line => innerIndent + line.trimStart()).join('\n')}\n${indent}}`;

        await editor.edit(editBuilder => {
          editBuilder.replace(selection, wrapped);
        });
      }
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('naze.extractComponent', async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) return;

      const selection = editor.selection;
      const text = editor.document.getText(selection);

      if (!text) {
        vscode.window.showWarningMessage('Please select code to extract');
        return;
      }

      const componentName = await vscode.window.showInputBox({
        prompt: 'Enter component name',
        placeHolder: 'MyComponent',
        validateInput: (value) => {
          if (!value || !/^[A-Z][a-zA-Z0-9]*$/.test(value)) {
            return 'Component name must start with uppercase letter';
          }
          return null;
        }
      });

      if (!componentName) return;

      // Create the component definition
      const componentDef = `component ${componentName}() {\n${text.split('\n').map(line => '  ' + line).join('\n')}\n}\n\n`;

      await editor.edit(editBuilder => {
        // Insert component at the top of the file (after any imports)
        const firstLine = editor.document.lineAt(0);
        editBuilder.insert(firstLine.range.start, componentDef);

        // Replace selection with component usage
        editBuilder.replace(selection, `${componentName}()`);
      });
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('naze.addClosingBrace', async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) return;

      const position = editor.selection.active;
      await editor.edit(editBuilder => {
        editBuilder.insert(position, '\n}');
      });
    })
  );

  console.log('Naze extension activated');
}

async function startLanguageServer(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration('naze');

  // Determine the path to the LSP binary
  let serverPath = config.get<string>('lsp.path');

  if (!serverPath) {
    // Use bundled binary (platform-specific)
    const platform = process.platform;
    const arch = process.arch;
    const ext = platform === 'win32' ? '.exe' : '';
    serverPath = context.asAbsolutePath(
      path.join('bin', `naze-lsp-${platform}-${arch}${ext}`)
    );
  }

  // Server options - run the naze-lsp binary
  const serverOptions: ServerOptions = {
    run: {
      command: serverPath,
      transport: TransportKind.stdio,
    },
    debug: {
      command: serverPath,
      transport: TransportKind.stdio,
    },
  };

  // Client options
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'naze' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.naze'),
    },
  };

  // Create and start the client
  client = new LanguageClient(
    'naze-lsp',
    'Naze Language Server',
    serverOptions,
    clientOptions
  );

  try {
    await client.start();
    console.log('Naze language server started');
  } catch (error) {
    console.error('Failed to start Naze language server:', error);
    vscode.window.showErrorMessage(
      `Failed to start Naze language server: ${error}`
    );
  }
}

export function deactivate(): Thenable<void> | undefined {
  if (client) {
    return client.stop();
  }
  return undefined;
}
