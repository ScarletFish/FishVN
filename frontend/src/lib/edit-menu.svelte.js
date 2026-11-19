import { copyText, readClip } from './clipboard.js';
import { CONTENT_SELECTOR, FIELD_SELECTOR } from './selection.js';
import { t } from './i18n.js';
import { toast } from './toast.svelte.js';

const BLOCK = 'p, li, dd, dt, h1, h2, h3, h4, h5, h6, blockquote, pre, td, th';
const MARGIN = 8;

let menu = $state(null);
let targetEl = null;

export function currentEditMenu() {
  return menu;
}

export function closeEditMenu() {
  menu = null;
  targetEl = null;
}

export function placeMenu(x, y, width, height, viewport) {
  return {
    left: Math.min(Math.max(MARGIN, x), Math.max(MARGIN, viewport.width - width - MARGIN)),
    top: Math.min(Math.max(MARGIN, y), Math.max(MARGIN, viewport.height - height - MARGIN)),
  };
}

function editableTarget(el) {
  const hit = el.closest?.(FIELD_SELECTOR);
  return hit instanceof HTMLElement ? hit : null;
}

function fieldSelection(el) {
  const range = fieldRange(el);
  if (range) return el.value.slice(range.start, range.end);
  if (el.isContentEditable) {
    const sel = window.getSelection();
    return sel && !sel.isCollapsed ? sel.toString() : '';
  }
  return '';
}

function fieldRange(el) {
  if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement) {
    const start = el.selectionStart;
    const end = el.selectionEnd;
    if (start == null || end == null || start === end) return null;
    return { start, end };
  }
  return null;
}

export function openEditMenuAt(event) {
  const el = event.target instanceof Element ? event.target : null;
  if (!el) return false;
  const editable = editableTarget(el);
  const inContent = Boolean(el.closest(CONTENT_SELECTOR));
  if (!editable && !inContent) return false;
  const text = editable ? fieldSelection(editable) : (window.getSelection()?.toString() ?? '');
  const hasSelection = Boolean(text.trim());
  if (!editable && !hasSelection) return false;
  targetEl = editable ?? el;
  menu = {
    x: event.clientX,
    y: event.clientY,
    kind: editable ? 'editable' : 'text',
    hasSelection,
    text,
    range: editable ? fieldRange(editable) : null,
  };
  return true;
}

function focusTarget(el) {
  if (el instanceof HTMLElement) el.focus({ preventScroll: true });
}

function selectAllIn(el) {
  if (!el) return;
  if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement) {
    focusTarget(el);
    el.select();
    return;
  }
  const scope = el.isContentEditable ? el : (el.closest?.(BLOCK) ?? el);
  if (!(scope instanceof HTMLElement)) return;
  focusTarget(el);
  const range = document.createRange();
  range.selectNodeContents(scope);
  const sel = window.getSelection();
  sel?.removeAllRanges();
  sel?.addRange(range);
}

export async function runEditAction(id) {
  const el = targetEl;
  const text = menu?.text ?? '';
  const range = menu?.range ?? null;
  closeEditMenu();
  if ((id === 'cut' || id === 'copy') && !text) return;
  try {
    if (id === 'selectAll') {
      selectAllIn(el);
      return;
    }
    if (id === 'cut') {
      focusTarget(el);
      if (range && el?.setSelectionRange) el.setSelectionRange(range.start, range.end);
      document.execCommand('cut');
      return;
    }
    if (id === 'copy') {
      await copyText(text);
      toast(t('toast.copied'), 'success', { duration: 1500 });
      return;
    }
    if (id === 'paste') {
      const clip = await readClip();
      if (!clip) return;
      focusTarget(el);
      if (range && el?.setSelectionRange) el.setSelectionRange(range.start, range.end);
      document.execCommand('insertText', false, clip);
    }
  } catch {
    toast(t('editMenu.failed'), 'error');
  }
}
