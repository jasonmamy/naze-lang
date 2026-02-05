import { useEffect, useRef, useState } from 'react';
import './LivePreview.css';

interface LivePreviewProps {
  content: string;
}

export function LivePreview({ content }: LivePreviewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [error, setError] = useState<string | null>(null);
  const [scale, setScale] = useState(1);

  // Simple preview renderer
  // In a full implementation, this would use naze-layout + naze-renderer WASM
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    try {
      // Clear canvas
      ctx.fillStyle = '#1e1e1e';
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      // Simple preview - render basic structure
      renderPreview(ctx, content);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Preview error');
    }
  }, [content]);

  return (
    <div className="live-preview">
      <div className="preview-header">
        <span className="preview-title">Preview</span>
        <div className="preview-controls">
          <button
            className="zoom-btn"
            onClick={() => setScale(s => Math.max(0.25, s - 0.25))}
          >
            −
          </button>
          <span className="zoom-level">{Math.round(scale * 100)}%</span>
          <button
            className="zoom-btn"
            onClick={() => setScale(s => Math.min(2, s + 0.25))}
          >
            +
          </button>
        </div>
      </div>

      <div className="preview-viewport">
        {error ? (
          <div className="preview-error">
            <span className="error-icon">⚠️</span>
            <span>{error}</span>
          </div>
        ) : (
          <div
            className="preview-canvas-wrapper"
            style={{ transform: `scale(${scale})` }}
          >
            <canvas
              ref={canvasRef}
              width={375}
              height={667}
              className="preview-canvas"
            />
          </div>
        )}
      </div>

      <div className="preview-footer">
        <span className="device-info">iPhone SE (375 × 667)</span>
      </div>
    </div>
  );
}

/**
 * Simple preview renderer.
 * This is a placeholder - the real implementation would use WASM.
 */
function renderPreview(ctx: CanvasRenderingContext2D, content: string) {
  const width = ctx.canvas.width;
  const height = ctx.canvas.height;

  // Parse simple elements and render
  const lines = content.split('\n');
  let y = 20;

  // Background
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, width, height);

  for (const line of lines) {
    const trimmed = line.trim();

    // Skip empty lines and comments
    if (!trimmed || trimmed.startsWith('--')) continue;

    // Detect element types and render simple representations
    if (trimmed.startsWith('heading ')) {
      const match = trimmed.match(/heading\s+"([^"]+)"/);
      if (match) {
        ctx.fillStyle = '#000000';
        ctx.font = 'bold 24px sans-serif';
        ctx.fillText(match[1], 20, y + 24);
        y += 40;
      }
    } else if (trimmed.startsWith('text ')) {
      const match = trimmed.match(/text\s+"([^"]+)"/);
      if (match) {
        ctx.fillStyle = '#333333';
        ctx.font = '14px sans-serif';
        ctx.fillText(match[1], 20, y + 14);
        y += 24;
      }
    } else if (trimmed.startsWith('rect ')) {
      // Extract color if present
      const colorMatch = trimmed.match(/color:\s*(#[0-9a-fA-F]+)/);
      const widthMatch = trimmed.match(/width:\s*(\d+)/);
      const heightMatch = trimmed.match(/height:\s*(\d+)/);
      const radiusMatch = trimmed.match(/radius:\s*(\d+)/);

      const rectColor = colorMatch ? colorMatch[1] : '#cccccc';
      const rectWidth = widthMatch ? parseInt(widthMatch[1]) : 100;
      const rectHeight = heightMatch ? parseInt(heightMatch[1]) : 50;
      const radius = radiusMatch ? parseInt(radiusMatch[1]) : 0;

      ctx.fillStyle = rectColor;

      if (radius > 0) {
        roundRect(ctx, 20, y, rectWidth, rectHeight, radius);
        ctx.fill();
      } else {
        ctx.fillRect(20, y, rectWidth, rectHeight);
      }

      y += rectHeight + 10;
    } else if (trimmed.startsWith('input ')) {
      // Render input placeholder
      ctx.strokeStyle = '#cccccc';
      ctx.lineWidth = 1;
      ctx.strokeRect(20, y, width - 40, 36);

      const placeholderMatch = trimmed.match(/placeholder:\s*"([^"]+)"/);
      if (placeholderMatch) {
        ctx.fillStyle = '#999999';
        ctx.font = '14px sans-serif';
        ctx.fillText(placeholderMatch[1], 28, y + 23);
      }

      y += 46;
    } else if (trimmed.startsWith('app ') || trimmed.startsWith('column ') || trimmed.startsWith('row ')) {
      // Container elements - just add some padding
      const paddingMatch = trimmed.match(/padding:\s*(\d+)/);
      if (paddingMatch) {
        y += parseInt(paddingMatch[1]) / 2;
      }
    }
  }
}

function roundRect(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number
) {
  ctx.beginPath();
  ctx.moveTo(x + radius, y);
  ctx.lineTo(x + width - radius, y);
  ctx.quadraticCurveTo(x + width, y, x + width, y + radius);
  ctx.lineTo(x + width, y + height - radius);
  ctx.quadraticCurveTo(x + width, y + height, x + width - radius, y + height);
  ctx.lineTo(x + radius, y + height);
  ctx.quadraticCurveTo(x, y + height, x, y + height - radius);
  ctx.lineTo(x, y + radius);
  ctx.quadraticCurveTo(x, y, x + radius, y);
  ctx.closePath();
}
