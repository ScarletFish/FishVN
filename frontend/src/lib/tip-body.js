export function createTipBody() {
  const root = document.createElement('div');
  root.className = 'tip-body';

  const label = document.createElement('span');
  label.className = 'tip-label';

  const key = document.createElement('span');
  key.className = 'tip-key';
  key.hidden = true;

  root.append(label, key);

  return {
    root,
    set(text, keyText) {
      label.textContent = text ?? '';
      key.textContent = keyText ?? '';
      key.hidden = !keyText;
    },
  };
}
