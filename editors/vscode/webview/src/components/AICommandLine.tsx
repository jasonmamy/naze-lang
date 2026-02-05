import { useState, useEffect, useRef } from 'react';
import './AICommandLine.css';

interface AICommandLineProps {
  onSubmit: (prompt: string) => void;
}

interface AIMessage {
  type: 'user' | 'assistant' | 'error';
  content: string;
}

export function AICommandLine({ onSubmit }: AICommandLineProps) {
  const [prompt, setPrompt] = useState('');
  const [messages, setMessages] = useState<AIMessage[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Handle AI responses from extension
  useEffect(() => {
    const handleAIResponse = (e: CustomEvent<{ content?: string; error?: string }>) => {
      setIsLoading(false);
      if (e.detail.error) {
        setMessages(prev => [...prev, { type: 'error', content: e.detail.error! }]);
      } else if (e.detail.content) {
        setMessages(prev => [...prev, { type: 'assistant', content: e.detail.content! }]);
      }
    };

    window.addEventListener('ai-response', handleAIResponse as EventListener);
    return () => window.removeEventListener('ai-response', handleAIResponse as EventListener);
  }, []);

  // Scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!prompt.trim() || isLoading) return;

    setMessages(prev => [...prev, { type: 'user', content: prompt }]);
    setIsLoading(true);
    setIsExpanded(true);
    onSubmit(prompt);
    setPrompt('');
  };

  const handleVoiceClick = () => {
    // Voice input is stubbed as "Coming Soon"
    alert('Voice input coming soon! This feature will allow you to speak your commands.');
  };

  const examplePrompts = [
    'Add a login form with email and password',
    'Create a navigation bar with logo and links',
    'Make this button blue with rounded corners',
    'Add a card grid layout with images',
  ];

  return (
    <div className={`ai-command-line ${isExpanded ? 'expanded' : ''}`}>
      <div className="ai-header" onClick={() => setIsExpanded(!isExpanded)}>
        <span className="ai-icon">🤖</span>
        <span className="ai-title">AI Assistant</span>
        <button
          className="voice-btn"
          onClick={(e) => {
            e.stopPropagation();
            handleVoiceClick();
          }}
          title="Voice input - Coming Soon"
        >
          🎤
          <span className="coming-soon-badge">Soon</span>
        </button>
        <span className="expand-icon">{isExpanded ? '▼' : '▲'}</span>
      </div>

      {isExpanded && (
        <div className="ai-content">
          {messages.length > 0 ? (
            <div className="ai-messages">
              {messages.map((msg, i) => (
                <div key={i} className={`ai-message ${msg.type}`}>
                  {msg.type === 'user' && <span className="message-icon">👤</span>}
                  {msg.type === 'assistant' && <span className="message-icon">🤖</span>}
                  {msg.type === 'error' && <span className="message-icon">⚠️</span>}
                  <div className="message-content">
                    {msg.type === 'assistant' ? (
                      <pre className="code-block">{msg.content}</pre>
                    ) : (
                      msg.content
                    )}
                  </div>
                  {msg.type === 'assistant' && (
                    <div className="message-actions">
                      <button className="action-btn">Insert at cursor</button>
                      <button className="action-btn secondary">Copy</button>
                    </div>
                  )}
                </div>
              ))}
              {isLoading && (
                <div className="ai-message assistant loading">
                  <span className="message-icon">🤖</span>
                  <div className="loading-dots">
                    <span>.</span><span>.</span><span>.</span>
                  </div>
                </div>
              )}
              <div ref={messagesEndRef} />
            </div>
          ) : (
            <div className="ai-examples">
              <p className="examples-label">Try asking:</p>
              <div className="example-prompts">
                {examplePrompts.map((example, i) => (
                  <button
                    key={i}
                    className="example-prompt"
                    onClick={() => {
                      setPrompt(example);
                      inputRef.current?.focus();
                    }}
                  >
                    {example}
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      <form className="ai-input-form" onSubmit={handleSubmit}>
        <input
          ref={inputRef}
          type="text"
          className="ai-input"
          placeholder="Ask AI to create, modify, or explain..."
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          disabled={isLoading}
        />
        <button
          type="submit"
          className="ai-submit"
          disabled={!prompt.trim() || isLoading}
        >
          {isLoading ? '...' : 'Send'}
        </button>
      </form>
    </div>
  );
}
