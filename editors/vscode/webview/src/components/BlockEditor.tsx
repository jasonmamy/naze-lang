import { AstBlock } from '../types';
import { Block } from './Block';
import './BlockEditor.css';

interface BlockEditorProps {
  ast: AstBlock;
  selectedBlockId: string | null;
  source: string;
  onSelectBlock: (id: string | null) => void;
  onEdit: (edit: { start: number; end: number; text: string }) => void;
}

export function BlockEditor({
  ast,
  selectedBlockId,
  source,
  onSelectBlock,
  onEdit,
}: BlockEditorProps) {
  return (
    <div className="block-editor">
      <Block
        block={ast}
        depth={0}
        selectedBlockId={selectedBlockId}
        source={source}
        onSelectBlock={onSelectBlock}
        onEdit={onEdit}
      />
    </div>
  );
}
