import { useState, useEffect, useCallback } from 'react';
import { VsCodeApi, DocumentState, ExtensionMessage, AstBlock } from './types';
import { parseNazeToAst } from './parser';
import { Toolbar } from './components/Toolbar';
import { BlockEditor } from './components/BlockEditor';
import { PropertiesPanel } from './components/PropertiesPanel';
import { AICommandLine } from './components/AICommandLine';
import { LivePreview } from './components/LivePreview';
import './App.css';

// Acquire VS Code API
const vscode: VsCodeApi = window.acquireVsCodeApi();

export default function App() {
  const [document, setDocument] = useState<DocumentState>({
    fileName: '',
    content: '',
    ast: null,
    parseError: null,
  });
  const [selectedBlockId, setSelectedBlockId] = useState<string | null>(null);
  const [showPreview, setShowPreview] = useState(true);

  // Handle messages from the extension
  useEffect(() => {
    const handleMessage = (event: MessageEvent<ExtensionMessage>) => {
      const message = event.data;
      switch (message.type) {
        case 'update': {
          const { ast, error } = parseNazeToAst(message.content);
          setDocument({
            fileName: message.fileName,
            content: message.content,
            ast,
            parseError: error,
          });
          break;
        }
        case 'ai-response':
          // Handle AI response in AICommandLine component
          window.dispatchEvent(new CustomEvent('ai-response', { detail: message }));
          break;
      }
    };

    window.addEventListener('message', handleMessage);

    // Notify extension that we're ready
    vscode.postMessage({ type: 'ready' });

    return () => window.removeEventListener('message', handleMessage);
  }, []);

  // Find selected block in AST
  const findBlock = useCallback((ast: AstBlock | null, id: string): AstBlock | null => {
    if (!ast) return null;
    if (ast.id === id) return ast;
    for (const child of ast.children) {
      const found = findBlock(child, id);
      if (found) return found;
    }
    return null;
  }, []);

  const selectedBlock = selectedBlockId ? findBlock(document.ast, selectedBlockId) : null;

  // Apply edit to source
  const applyEdit = useCallback((edit: { start: number; end: number; text: string }) => {
    vscode.postMessage({ type: 'edit', edit });
  }, []);

  // Handle AI request
  const handleAIRequest = useCallback((prompt: string) => {
    const context = selectedBlock
      ? `Selected element: ${selectedBlock.type}${selectedBlock.name ? ` "${selectedBlock.name}"` : ''}`
      : undefined;
    vscode.postMessage({ type: 'ai-request', prompt, context });
  }, [selectedBlock]);

  return (
    <div className="app">
      <Toolbar
        fileName={document.fileName}
        showPreview={showPreview}
        onTogglePreview={() => setShowPreview(!showPreview)}
      />

      <div className="main-content">
        <div className="editor-area">
          {document.parseError ? (
            <div className="parse-error">
              <h3>Parse Error</h3>
              <pre>{document.parseError}</pre>
            </div>
          ) : document.ast ? (
            <BlockEditor
              ast={document.ast}
              selectedBlockId={selectedBlockId}
              source={document.content}
              onSelectBlock={setSelectedBlockId}
              onEdit={applyEdit}
            />
          ) : (
            <div className="empty-state">
              <p>Open a .naze file to start editing</p>
            </div>
          )}
        </div>

        {selectedBlock && (
          <PropertiesPanel
            block={selectedBlock}
            source={document.content}
            onEdit={applyEdit}
            onClose={() => setSelectedBlockId(null)}
          />
        )}

        {showPreview && (
          <LivePreview content={document.content} />
        )}
      </div>

      <AICommandLine onSubmit={handleAIRequest} />
    </div>
  );
}
