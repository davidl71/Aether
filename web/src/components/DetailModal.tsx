import type { ReactNode } from 'react';

interface DetailModalProps {
  title: string;
  onClose: () => void;
  children: ReactNode;
}

export function DetailModal({ title, onClose, children }: DetailModalProps) {
  return (
    <div className="modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
      <div className="modal__backdrop" onClick={onClose} />
      <div className="modal__content">
        <header className="modal__header">
          <h3 id="modal-title">{title}</h3>
          <button type="button" className="modal__close" onClick={onClose} aria-label="Close">
            ×
          </button>
        </header>
        <div className="modal__body">{children}</div>
        <footer className="modal__footer">
          <button type="button" className="btn btn--primary" onClick={onClose}>
            Close
          </button>
        </footer>
      </div>
    </div>
  );
}
