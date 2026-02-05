import './Toolbar.css';

interface ToolbarProps {
  fileName: string;
  showPreview: boolean;
  onTogglePreview: () => void;
}

export function Toolbar({ fileName, showPreview, onTogglePreview }: ToolbarProps) {
  const displayName = fileName ? fileName.split(/[\\/]/).pop() : 'Untitled';

  return (
    <div className="toolbar">
      <span className="toolbar-icon">📱</span>
      <span className="toolbar-title">{displayName}</span>
      <div className="toolbar-actions">
        <button
          className={`toolbar-btn ${showPreview ? 'active' : ''}`}
          onClick={onTogglePreview}
          title={showPreview ? 'Hide Preview' : 'Show Preview'}
        >
          {showPreview ? '👁️ Preview' : '👁️‍🗨️ Preview'}
        </button>
      </div>
    </div>
  );
}
